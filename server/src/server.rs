use crate::client_manager::ClientManager;
use crate::tun_device::TunDevice;
use crate::{error, handler};
use log::info;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;

pub struct Server {
    socket: Arc<UdpSocket>,
    tun_device: TunDevice,
    client_manager: ClientManager,
}

impl Server {
    pub async fn new(server_addr: SocketAddr) -> error::Result<Self> {
        let socket = UdpSocket::bind(server_addr).await?;
        let socket = Arc::new(socket);

        info!("UDP socket bound to {}", server_addr);

        let tun_device = TunDevice::new().await?;

        Ok(Self {
            socket,
            tun_device,
            client_manager: ClientManager::new(),
        })
    }

    pub async fn run(self) -> error::Result<()> {
        info!("VPN server (build 0.0.2) is running... ");
        
        let socket = self.socket.clone();
        let client_manager = self.client_manager.clone();

        let (tun_tx, tun_rx) = mpsc::channel(3072);
        let tun_device = self.tun_device;

        let udp_handler = handler::udp::handle(socket, client_manager.clone(), tun_tx);
        let tun_handler = handler::tun::handle(tun_device, client_manager, tun_rx);

        tokio::try_join!(udp_handler, tun_handler)?;

        Ok(())
    }
}
