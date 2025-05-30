mod client_manager;
mod error;
pub(crate) mod handler;
mod server;
mod simple_impl;
mod tun_device;

use crate::server::Server;
use clap::Parser;
use std::net::SocketAddr;
use log::info;
//cargo build --release && sudo ./target/release/server

#[tokio::main]
async fn main() {
    unsafe {
        std::env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();

    info!("(build 0.0.7)");
    let args = Args::parse();

    let server_addr: SocketAddr = format!("0.0.0.0:{}", args.port)
        .parse()
        .expect("Invalid server port");
    
    let server = match Server::new(server_addr).await {
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