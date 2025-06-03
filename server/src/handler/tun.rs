use crate::client_manager::ClientManager;
use crate::tun_device::TunDevice;
use log::error;

///Обработка TUN
pub(crate) fn handle(
    mut tun_device: TunDevice,
    client_manager: ClientManager,
    mut rx: tokio::sync::mpsc::Receiver<Vec<u8>>,
    key: String,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut buf = vec![0u8; 3072];
        loop {
            tokio::select! {
                result = rx.recv() => {
                    let packet = result.unwrap();

                    match tun_device.write_packet(&packet).await {
                        Ok(()) => {}
                        Err(err) => {
                            error!("TUN write error: {}", err);
                        }
                    }
                }
                result = tun_device.read_packet(&mut buf) => {
                    match result {
                        Ok(0) => {
                            error!("TUN interface closed");
                            break;
                        }
                        Ok(n) => {
                            let packet = &buf[..n];
                            let packet = match protocol::encrypt(packet, key.as_bytes()) {
                                Ok(packet) => packet,
                                Err(err) => {
                                    error!("Failed to encrypt packet from TUN: {}", err);
                                    continue;
                                }
                            };

                            for (addr, socket) in client_manager.get_clients().iter() {
                                if let Err(e) = socket.send_to(&packet, addr).await {
                                    error!("Failed to send packet to {}: {}", addr, e);
                                    client_manager.remove_client(addr);
                                }
                            }
                        }
                        Err(err) => {
                            error!("Read error with TUN: {}", err);
                            break;
                        }
                    }
                }

            }
        }
    })
}
