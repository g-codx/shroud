use serde::{Deserialize, Serialize};
use crate::error;

#[derive(Serialize, Deserialize, Debug)]
pub struct Packet {
    pub nonce: Vec<u8>,
    pub data: Vec<u8>,
}

impl Packet {
    pub fn encode(self) -> error::Result<Vec<u8>> {
        Ok(bincode::serialize(&self)?)
    }

    pub fn decode(data: &[u8]) -> error::Result<Self> {
        Ok(bincode::deserialize(data)?)
    }
}