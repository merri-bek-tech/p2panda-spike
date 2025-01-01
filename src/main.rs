use anyhow::Result;
use gethostname::gethostname;
use p2panda_core::{Hash, PrivateKey};
use p2panda_discovery::mdns::LocalDiscovery;
use p2panda_net::network::{FromNetwork, ToNetwork};
use p2panda_net::{NetworkBuilder, NetworkId, TopicId};
use p2panda_sync::TopicQuery;
use serde::{Deserialize, Serialize};
use sites::Sites;
use std::env;

mod messages;
mod site_messages;
mod sites;

use messages::Message;
use site_messages::{SiteMessages, SiteRegistration};

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
    let mut sites = Sites::build();

    let site_name = get_site_name();
    println!("Starting client for site: {}", site_name);

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
            handle_network_event(event, &mut sites);
        }
    });

    println!(".. waiting for peers to join ..");
    let _ = ready.await;

    println!("found other peers, you're ready to chat!");

    // spawn a task to announce the site every 30 seconds
    tokio::task::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            announce_site(&private_key, &site_name, &tx).await.ok();
        }
    });

    tokio::signal::ctrl_c().await?;

    network.shutdown().await?;

    Ok(())
}

async fn announce_site(
    private_key: &PrivateKey,
    name: &str,
    tx: &tokio::sync::mpsc::Sender<ToNetwork>,
) -> Result<()> {
    println!("Announcing myself: {}", name);
    tx.send(ToNetwork::Message {
        bytes: Message::sign_and_encode(
            private_key,
            SiteMessages::SiteRegistration(SiteRegistration {
                name: name.to_string(),
            }),
        )?,
    })
    .await?;
    Ok(())
}

fn handle_network_event(event: FromNetwork, sites: &mut Sites) {
    match event {
        FromNetwork::GossipMessage { bytes, .. } => match Message::decode_and_verify(&bytes) {
            Ok(message) => {
                handle_message(message, sites);
            }
            Err(err) => {
                eprintln!("Invalid gossip message: {}", err);
            }
        },
        _ => panic!("no sync messages expected"),
    }
}

fn handle_message(message: Message<SiteMessages>, sites: &mut Sites) {
    match message.payload {
        SiteMessages::SiteRegistration(registration) => {
            println!("Received SiteRegistration: {:?}", registration);
            sites.register(registration.name);
            sites.log();
        }
        SiteMessages::SiteNotification(notification) => {
            println!("Received SiteNotification: {:?}", notification);
        }
    }
}

fn get_site_name() -> String {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        return args[1].to_string();
    }

    gethostname().to_string_lossy().to_string()
}
