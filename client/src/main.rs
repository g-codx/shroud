mod config;
mod error;

use crate::config::ClientConfig;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UdpSocket;
use tokio::task;
use tun::{AbstractDevice, AsyncDevice, Configuration};
//cargo build --bin client --release && sudo ./target/release/client
//cargo build --release && sudo ./target/release/client

const TUN_NAME: &str = "shroud-tun";

#[tokio::main]
async fn main() -> error::Result<()> {
    let cfg = ClientConfig::load()?;
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    let server_addr = format!("{}:{}", cfg.server_ip, cfg.server_port);
    let socket_port = socket
        .local_addr()
        .expect("could not get local socket address")
        .port();

    // Конфигурация TUN интерфейса
    let mut config = Configuration::default();
    config
        .tun_name(TUN_NAME) // имя интерфейса (можно пустое для автогенерации)
        .address("10.0.0.2") // IP клиента
        .netmask("255.255.255.0")
        .destination("10.0.0.1")
        .mtu(1500)
        .up(); // поднять интерфейс

    // Создаём асинхронный TUN-девайс
    let mut tun: AsyncDevice = tun::create_as_async(&config)?;
    println!("TUN интерфейс создан: {}", tun.tun_name()?);

    task::spawn(async move {
        println!("NETWORK SETUP STARTED 2.0");
        tokio::time::sleep(Duration::from_secs(5)).await;
        network::setup_network(socket_port, TUN_NAME);
        println!("NETWORK READY");
    });
    // Буферы для передачи
    let mut tun_buf = [0u8; 3000];
    let mut sock_buf = [0u8; 3000];

    loop {
        tokio::select! {
            // Читаем пакет из TUN и отправляем в сокет
            n = tun.read(&mut tun_buf) => {
                let n = n?;
                if n == 0 {
                    println!("TUN интерфейс закрылся");
                    break;
                }

                let packet = &tun_buf[..n];
                let packet = protocol::encrypt(packet, cfg.key.as_bytes())?;
                socket.send_to(&packet, server_addr.as_str()).await?;
            }
            // Читаем пакет из сокета и пишем в TUN
            res = socket.recv_from(&mut sock_buf) => {
                let (n, _addr) = res?;
                if n == 0 {
                    println!("Сервер закрыл соединение");
                    break;
                }
                let packet = protocol::decrypt(&sock_buf[..n], cfg.key.as_bytes())?;
                tun.write_all(&packet).await?;
            }
            sig = tokio::signal::ctrl_c() => {
                sig?;
                println!("NETWORK CLEAN STARTED");
                network::cleanup_vpn_rules(socket_port);
                println!("NETWORK CLEAN READY");
                break;
            }
        }
    }

    Ok(())
}
