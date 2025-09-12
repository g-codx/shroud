mod config;
#[cfg(target_os = "linux")]
mod linux;
mod proxy;
#[cfg(target_os = "windows")]
mod windows;

use config::AppConfig;
use std::fs;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // let config_path = Path::new("./config.json");
    // let config_str = fs::read_to_string(config_path).unwrap();
    // let config: AppConfig = serde_json::from_str(&config_str).unwrap();

    #[cfg(target_os = "linux")]
    linux::run_linux().await?;

    #[cfg(target_os = "windows")]
    windows::run_windows(config).await?;

    Ok(())
}
