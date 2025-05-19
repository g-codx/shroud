mod error;
mod mini_cli;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UdpSocket;
use tokio::process::Command;
use tun::{AbstractDevice, AsyncDevice, Configuration};

//cargo build --bin client --release && sudo ./target/release/client

//cargo build --release && sudo ./target/release/client

#[tokio::main]
async fn main() -> error::Result<()> {
    // Конфигурация TUN интерфейса
    let mut config = Configuration::default();
    config
        .tun_name("tun0") // имя интерфейса (можно пустое для автогенерации)
        .address((10, 8, 0, 2)) // IP клиента
        .netmask((255, 255, 255, 0))
        .destination((10, 8, 0, 1))
        .mtu(1500)
        .up(); // поднять интерфейс

    // Создаём асинхронный TUN-девайс
    let mut tun: AsyncDevice = tun::create_as_async(&config)?;
    println!("TUN интерфейс создан: {}", tun.tun_name()?);

    let route_output = Command::new("ip")
        .arg("route")
        .arg("add")
        .arg("0.0.0.0/0")
        .arg("via")
        .arg("10.8.0.1")
        .arg("dev")
        .arg("tun0")
        .output()
        .await
        .expect("Failed to execute IP ROUTE command");

    if !route_output.status.success() {
        eprintln!(
            "Failed to set route: {}",
            String::from_utf8_lossy(&route_output.stderr)
        );
    }

    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    let server_addr = "192.168.0.103:44444"; //79.133.182.111:44444

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
                let packet = protocol::encrypt(packet)?;
                // println!("Принят и зашифрован пакет из TUN");
                socket.send_to(&packet, server_addr).await?;
                // println!("Зашифрованный пакет отправлен на сервер");
            }

            // Читаем пакет из сокета и пишем в TUN
            res = socket.recv_from(&mut sock_buf) => {
                let (n, _addr) = res?;
                if n == 0 {
                    println!("Сервер закрыл соединение");
                    break;
                }
                let packet = protocol::decrypt(&sock_buf[..n])?;
                println!("Принят пакет от сервера и расшифрован");
                tun.write_all(&packet).await?;
                println!("Расшифрованный пакет от сервера отправлен в TUN");
            }
        }
    }

    Ok(())
}
