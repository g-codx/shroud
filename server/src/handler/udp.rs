use crate::client_manager::ClientManager;
use log::{error, info};
use std::sync::Arc;
use tokio::net::UdpSocket;

///Обработка входящих UDP пакетов
pub(crate) fn handle(
    socket: Arc<UdpSocket>,
    client_manager: ClientManager,
    tx: tokio::sync::mpsc::Sender<Vec<u8>>,
    key: String
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut buf = vec![0u8; 3072];
        loop {
            match socket.recv_from(&mut buf).await {
                Ok((size, addr)) => {
                    let clients = client_manager.get_clients();

                    if !clients.iter().any(|(a, _)| a == &addr) {
                        info!("New client connected: {}", addr);
                        client_manager.add_client(addr, socket.clone());
                    }

                    let raw_packet = &buf[..size];
                    let packet = match protocol::decrypt(raw_packet, key.as_bytes()) {
                        Ok(packet) => packet,
                        Err(err) => {
                            error!("Failed to decrypt packet for: {}, {}", addr, err);
                            continue;
                        }
                    };

                    tx.send(packet).await.unwrap();
                }
                Err(e) => {
                    error!("Error receiving from UDP socket: {}", e);
                }
            }
        }
    })
}
