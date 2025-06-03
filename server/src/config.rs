use serde::Deserialize;
use crate::error;

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub key: String,
    pub port: u16,
}

impl ServerConfig {
    pub fn load() -> error::Result<Self> {
        let cfg = include_str!("../config.toml");
        let loaded = toml::from_str::<ServerConfig>(cfg)?;
        Ok(loaded)
    }
}