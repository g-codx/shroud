mod client_manager;
mod config;
mod error;
pub(crate) mod handler;
mod server;
mod tun_device;

use crate::config::ServerConfig;
use crate::server::Server;
use clap::Parser;
use std::net::SocketAddr;
//cargo build --release && sudo ./target/release/server

#[tokio::main]
async fn main() {
    unsafe {
        std::env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();

    // let args = Args::parse();
    let cfg = ServerConfig::load().expect("Failed to load server config");

    let server_addr: SocketAddr = format!("0.0.0.0:{}", cfg.port)
        .parse()
        .expect("Invalid server port");

    let server = match Server::new(server_addr, cfg.key).await {
        Ok(s) => s,
        Err(err) => {
            panic!("Failed to start the server: {}", err);
        }
    };

    if let Some(err) = server.run().await.err() {
        panic!("Failed to start the server: {}", err);
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// UPD port
    #[arg(short, long)]
    port: String,
}
