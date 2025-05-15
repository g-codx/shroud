use crate::client_manager::ClientManager;
use log::{error, info};
use std::sync::Arc;
use tokio::net::UdpSocket;

pub(crate) fn handle(
    socket: Arc<UdpSocket>,
    client_manager: ClientManager,
    tx: tokio::sync::mpsc::Sender<Vec<u8>>,
) -> tokio::task::JoinHandle<()> {
    // Задача для обработки входящих UDP пакетов
    let udp_handler = tokio::spawn(async move {
        let mut buf = vec![0u8; 3072];
        loop {
            match socket.recv_from(&mut buf).await {
                Ok((size, addr)) => {
                    let clients = client_manager.get_clients();

                    // Если это новый клиент, добавляем его
                    if !clients.iter().any(|(a, _)| a == &addr) {
                        info!("New client connected: {}", addr);
                        client_manager.add_client(addr, socket.clone());
                    }

                    let raw_packet = &buf[..size];
                    let packet = match protocol::decrypt(raw_packet) {
                        Ok(packet) => packet,
                        Err(err) => {
                            error!("{}", err);
                            continue;
                        }
                    };

                    // info!("Получен и расшифрован пакет от клиента: {}", addr);

                    tx.send(packet).await.unwrap();

                    // info!("Пакет клиента отправлен в канал TUN");
                }
                Err(e) => {
                    error!("Error receiving from UDP socket: {}", e);
                }
            }
        }
    });

    udp_handler
}
