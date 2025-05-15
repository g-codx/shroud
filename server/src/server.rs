use crate::client_manager::ClientManager;
use crate::error;
use crate::tun_device::TunDevice;
use log::{error, info};
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

    pub async fn run(mut self) -> error::Result<()> {
        info!("VPN server is running...");

        let socket = self.socket.clone();
        let client_manager = self.client_manager.clone();

        //tun channel
        let (tx, mut rx) = mpsc::channel(3072);

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

        let tun_handler = tokio::spawn(async move {
            let mut buf = vec![0u8; 3072];
            loop {
                tokio::select! {
                    result = rx.recv() => {
                        let packet = result.unwrap();
                        // info!("Получен пакет от клиента из канала TUN");

                        match self.tun_device.write_packet(&packet).await {
                            Ok(()) => {
                                // info!("Пакет клиента отправлен в TUN");
                            }
                            Err(err) => {
                                error!("Ошибка записи в TUN: {}", err);
                            }
                        }
                    }
                    result = self.tun_device.read_packet(&mut buf) => {
                        match result {
                            Ok(0) => {
                                error!("TUN интерфейс закрылся");
                                break;
                            }
                            Ok(n) => {
                                let packet = &buf[..n];
                                let packet = match protocol::encrypt(packet) {
                                    Ok(packet) => packet,
                                    Err(err) => {
                                        error!("Не удалось зашифровать пакет из TUN: {}", err);
                                        continue;
                                    }
                                };

                                info!("Получен и зашифрован пакет из TUN");

                                // Отправляем пакет всем подключенным клиентам
                                for (addr, socket) in self.client_manager.get_clients().iter() {
                                    if let Err(e) = socket.send_to(&packet, addr).await {
                                        error!("Failed to send packet to {}: {}", addr, e);
                                        self.client_manager.remove_client(addr);
                                    }

                                    info!("Пакет отправлен клиентам...");
                                }
                            }
                            Err(err) => {
                                error!("Ошибка чтения с TUN: {}", err);
                                break;
                            }
                        }
                    }

                }
            }
        });

        tokio::try_join!(udp_handler, tun_handler)?;

        Ok(())
    }
}
