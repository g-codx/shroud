use crate::client_manager::ClientManager;
use crate::tun_device::TunDevice;
use log::{error, info};

pub(crate) fn handle(
    mut tun_device: TunDevice,
    client_manager: ClientManager,
    mut rx: tokio::sync::mpsc::Receiver<Vec<u8>>,
) -> tokio::task::JoinHandle<()> {
    let tun_handler = tokio::spawn(async move {
        let mut buf = vec![0u8; 3072];
        loop {
            tokio::select! {
                result = rx.recv() => {
                    let packet = result.unwrap();
                    // info!("Получен пакет от клиента из канала TUN");

                    match tun_device.write_packet(&packet).await {
                        Ok(()) => {
                            // info!("Пакет клиента отправлен в TUN");
                        }
                        Err(err) => {
                            error!("Ошибка записи в TUN: {}", err);
                        }
                    }
                }
                result = tun_device.read_packet(&mut buf) => {
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
                            for (addr, socket) in client_manager.get_clients().iter() {
                                if let Err(e) = socket.send_to(&packet, addr).await {
                                    error!("Failed to send packet to {}: {}", addr, e);
                                    client_manager.remove_client(addr);
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

    tun_handler
}
