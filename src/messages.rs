use anyhow::{bail, Result};
use p2panda_core::{PrivateKey, PublicKey, Signature};
use rand::random;
use serde::{Deserialize, Serialize};

use crate::site_messages::{SiteNotification, SiteRegistration};

#[derive(Serialize, Deserialize)]
pub enum Payload {
    SiteRegistration(SiteRegistration),
    SiteNotification(SiteNotification),
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    id: u32,
    signature: Signature,
    public_key: PublicKey,
    pub payload: Payload,
}

impl Message {
    pub fn sign_and_encode(private_key: &PrivateKey, payload: Payload) -> Result<Vec<u8>> {
        // Sign payload content
        let mut payload_bytes: Vec<u8> = Vec::new();
        ciborium::ser::into_writer(&payload, &mut payload_bytes)?;
        let signature = private_key.sign(&payload_bytes);

        // Encode message
        let message = Message {
            // Make every message unique, as duplicates get ignored during gossip broadcast
            id: random(),
            signature,
            public_key: private_key.public_key(),
            payload,
        };

        let mut message_bytes: Vec<u8> = Vec::new();
        ciborium::ser::into_writer(&message, &mut message_bytes)?;
        Ok(message_bytes)
    }

    pub fn decode_and_verify(bytes: &[u8]) -> Result<Message> {
        let message: Message = ciborium::de::from_reader(bytes)?;

        // Verify signature
        let mut payload_bytes: Vec<u8> = Vec::new();
        ciborium::ser::into_writer(&message.payload, &mut payload_bytes)?;
        if !message
            .public_key
            .verify(&payload_bytes, &message.signature)
        {
            bail!("Invalid signature");
        }

        Ok(message)
    }
}
