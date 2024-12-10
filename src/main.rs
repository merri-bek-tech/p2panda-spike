use p2panda_core::{Hash, PrivateKey};
use p2panda_discovery::mdns::LocalDiscovery;
use p2panda_net::{Network, NetworkBuilder, TopicId};
use p2panda_sync::TopicQuery;
use serde::{Deserialize, Serialize};

// The network can be used to automatically find and ask other peers about any data the
// application is interested in. This is expressed through "network-wide queries" over topics.
//
// In this example we would like to be able to query messages from each chat group, identified
// by a BLAKE3 hash.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
struct ChatGroup(Hash);

impl ChatGroup {
    pub fn new(name: &str) -> Self {
        Self(Hash::new(name.as_bytes()))
    }
}

impl TopicQuery for ChatGroup {}

impl TopicId for ChatGroup {
    fn id(&self) -> [u8; 32] {
        self.0.into()
    }
}

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    // Peers using the same "network id" will eventually find each other. This is the most global
    // identifier to group peers into multiple networks when necessary.
    let network_id: [u8; 32] = [1; 32];

    // Generate an Ed25519 private key which will be used to authenticate your peer towards others.
    let private_key: PrivateKey = PrivateKey::new();

    // Use mDNS to discover other peers on the local network.
    let mdns_discovery: LocalDiscovery =
        LocalDiscovery::new().expect("Failed to create local discovery");

    // Establish the p2p network which will automatically connect you to any discovered peers.
    let network: Network<ChatGroup> = NetworkBuilder::new(network_id)
        .private_key(private_key)
        .discovery(mdns_discovery)
        .build()
        .await
        .expect("Failed to build network");

    // From now on we can send and receive bytes to any peer interested in the same chat.
    let my_friends_group = ChatGroup::new("me-and-my-friends");
    let (_tx, mut _rx, _ready) = network
        .subscribe(my_friends_group)
        .await
        .expect("failed to subscribe");

    println!("Subscribed to chat group");
}
