use dashmap::DashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;

#[derive(Clone)]
pub struct ClientManager {
    clients: Arc<DashMap<SocketAddr, Arc<UdpSocket>>>,
}

impl ClientManager {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(DashMap::new()),
        }
    }

    pub fn add_client(&self, addr: SocketAddr, socket: Arc<UdpSocket>) {
        self.clients.insert(addr, socket);
    }

    pub fn remove_client(&self, addr: &SocketAddr) {
        self.clients.remove(addr);
    }

    pub fn get_clients(&self) -> Vec<(SocketAddr, Arc<UdpSocket>)> {
        self.clients
            .iter()
            .map(|entry| (*entry.key(), entry.value().clone()))
            .collect()
    }
}
