use std::collections::HashSet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub target_domains: HashSet<String>,
    pub socks5_proxy: String,
    pub interface_name: String,
}