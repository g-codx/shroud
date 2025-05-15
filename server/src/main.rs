mod client_manager;
mod error;
pub(crate) mod handler;
mod server;
mod simple_impl;
mod tun_device;

use crate::server::Server;
use std::net::SocketAddr;

//cargo build --release && sudo ./target/release/server

#[tokio::main]
async fn main() {
    unsafe {
        std::env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();

    let server_addr: SocketAddr = "0.0.0.0:3000".parse().unwrap();
    let vpn_server = Server::new(server_addr).await.unwrap();

    vpn_server.run().await.unwrap();
}
