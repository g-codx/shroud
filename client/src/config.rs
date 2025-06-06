use crate::error;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ClientConfig {
    pub key: String,
    pub server_ip: String,
    pub server_port: u16,
}

impl ClientConfig {
    pub fn load() -> error::Result<Self> {
        let cfg = include_str!("../config.toml");
        let loaded = toml::from_str::<ClientConfig>(cfg)?;
        Ok(loaded)
    }
}
