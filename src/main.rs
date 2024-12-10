use anyhow::{bail, Result};
use gethostname::gethostname;
use p2panda_core::{Hash, PrivateKey, PublicKey, Signature};
use p2panda_discovery::mdns::LocalDiscovery;
use p2panda_net::network::{FromNetwork, ToNetwork};
use p2panda_net::{NetworkBuilder, NetworkId, TopicId};
use p2panda_sync::TopicQuery;
use rand::random;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct ChatTopic(String, [u8; 32]);

impl ChatTopic {
    pub fn new(name: &str) -> Self {
        Self(name.to_owned(), *Hash::new(name).as_bytes())
    }
}

impl TopicQuery for ChatTopic {}

impl TopicId for ChatTopic {
    fn id(&self) -> [u8; 32] {
        self.1
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let site_name = get_site_name();

    let network_slug = "merri-bek.tech";
    let network_id: NetworkId = Hash::new(network_slug).into();

    let topic = ChatTopic::new("site_management");

    let private_key = PrivateKey::new();

    let network = NetworkBuilder::new(network_id)
        .discovery(LocalDiscovery::new()?)
        .build()
        .await?;

    let (tx, mut rx, ready) = network.subscribe(topic).await?;

    tokio::task::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                FromNetwork::GossipMessage { bytes, .. } => {
                    match Message::decode_and_verify(&bytes) {
                        Ok(message) => {
                            println!("Received {}: {}", message.public_key, message.text);
                        }
                        Err(err) => {
                            eprintln!("invalid gossip message: {err}");
                        }
                    }
                }
                _ => panic!("no sync messages expected"),
            }
        }
    });

    println!(".. waiting for peers to join ..");
    let _ = ready.await;

    println!("found other peers, you're ready to chat!");

    announce_site(&private_key, &site_name, &tx).await?;

    let (line_tx, mut line_rx) = mpsc::channel(1);
    std::thread::spawn(move || input_loop(line_tx));

    while let Some(text) = line_rx.recv().await {
        let bytes = Message::sign_and_encode(&private_key, &text)?;
        tx.send(ToNetwork::Message { bytes }).await.ok();
    }

    tokio::signal::ctrl_c().await?;

    network.shutdown().await?;

    Ok(())
}

#[derive(Serialize, Deserialize)]
struct Message {
    id: u32,
    signature: Signature,
    public_key: PublicKey,
    text: String,
}

impl Message {
    pub fn sign_and_encode(private_key: &PrivateKey, text: &str) -> Result<Vec<u8>> {
        // Sign text content
        let mut text_bytes: Vec<u8> = Vec::new();
        ciborium::ser::into_writer(text, &mut text_bytes)?;
        let signature = private_key.sign(&text_bytes);

        // Encode message
        let message = Message {
            // Make every message unique, as duplicates get ignored during gossip broadcast
            id: random(),
            signature,
            public_key: private_key.public_key(),
            text: text.to_owned(),
        };
        let mut bytes: Vec<u8> = Vec::new();
        ciborium::ser::into_writer(&message, &mut bytes)?;

        Ok(bytes)
    }

    fn decode_and_verify(bytes: &[u8]) -> Result<Self> {
        // Decode message
        let message: Self = ciborium::de::from_reader(bytes)?;

        // Verify signature
        let mut text_bytes: Vec<u8> = Vec::new();
        ciborium::ser::into_writer(&message.text, &mut text_bytes)?;
        if !message.public_key.verify(&text_bytes, &message.signature) {
            bail!("invalid signature");
        }

        Ok(message)
    }
}

fn input_loop(line_tx: mpsc::Sender<String>) -> Result<()> {
    let mut buffer = String::new();
    let stdin = std::io::stdin();
    loop {
        stdin.read_line(&mut buffer)?;
        line_tx.blocking_send(buffer.clone())?;
        buffer.clear();
    }
}

fn get_site_name() -> String {
    let hostname = gethostname();
    return hostname.to_string_lossy().to_string();
}

async fn announce_site(
    private_key: &PrivateKey,
    site_name: &str,
    tx: &tokio::sync::mpsc::Sender<ToNetwork>,
) -> Result<()> {
    println!("Announcing myself: {}", site_name);
    tx.send(ToNetwork::Message {
        bytes: Message::sign_and_encode(
            &private_key,
            &format!("{} has joined the chat", site_name),
        )?,
    })
    .await
    .ok();
    Ok(())
}
