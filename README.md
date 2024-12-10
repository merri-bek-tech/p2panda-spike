# P2Panda Spike for Merri-bek Tech

The goal of this prototype is to determine how the new modular P2Panda 2 could work for us for coordinating sites at [merri-bek.tech](https://merri-bek.tech).

## Usage

Run with `cargo run`. End by pressing CTRL-C. This command line app will announce itself as a site named after your computer's hostname.

If you want to use a different hostname, perhaps so you can run this twice on the same computer as a test, run as `cargo run -- otherhost`

## How does this work?

P2Panda is providing several functions.

Discovery: Nodes should find each other on the local network
Network: All nodes with the same `network_id` should communicate with each other and pass along unique messages.

However, P2Panda doesn't know or care about the message contents, it's just bytes. The messages used are encoded with [CBOR](https://cbor.io/) (Concise Binary Object Representation) which is essentially binary encoded json.

