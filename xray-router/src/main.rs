mod config;
mod proxy;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

use std::fs;
use std::path::Path;
use config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let config_path = Path::new("./config.json");
    let config_str = fs::read_to_string(config_path).unwrap();
    let config: AppConfig = serde_json::from_str(&config_str).unwrap();

    #[cfg(target_os = "linux")]
    linux::run_linux(config).await?;

    #[cfg(target_os = "windows")]
    windows::run_windows(config).await?;

    Ok(())
}