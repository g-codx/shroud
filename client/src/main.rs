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
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    let server_addr = "79.133.182.111:44444";
    //192.168.0.103:44444
    //79.133.182.111:44444

    // Конфигурация TUN интерфейса
    let mut config = Configuration::default();
    config
        .tun_name("tun0") // имя интерфейса (можно пустое для автогенерации)
        .address("10.0.0.2") // IP клиента
        .netmask("255.255.255.0")
        .destination("10.0.0.1")
        .mtu(1500)
        .up(); // поднять интерфейс

    // Создаём асинхронный TUN-девайс
    let mut tun: AsyncDevice = tun::create_as_async(&config)?;
    println!("TUN интерфейс создан: {}", tun.tun_name()?);

    setup_routing("tun0", server_addr).await?;
    println!("Route настроен: {}", tun.tun_name()?);

    // let route_output = Command::new("ip")
    //     .arg("route")
    //     .arg("add")
    //     .arg("0.0.0.0/0")
    //     .arg("via")
    //     .arg("10.8.0.1")
    //     .arg("dev")
    //     .arg("tun0")
    //     .output()
    //     .await
    //     .expect("Failed to execute IP ROUTE command");
    //
    // if !route_output.status.success() {
    //     eprintln!(
    //         "Failed to set route: {}",
    //         String::from_utf8_lossy(&route_output.stderr)
    //     );
    // }

    // Обеспечиваем очистку маршрутов при завершении
    // let cleanup = cleanup_routing("tun0", server_addr);
    // tokio::spawn(async move {
    //     tokio::signal::ctrl_c().await.ok();
    //     cleanup.await.ok();
    //     std::process::exit(0);
    // });

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

async fn setup_routing(tun_name: &str, server_ip: &str) -> error::Result<()> {
    //sudo ip route add 10.0.0.0/24 dev tun0
    let route_output = Command::new("ip")
        .arg("route")
        .arg("add")
        .arg("10.0.0.0/24")
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

    // // 1. Добавляем маршрут только для трафика к серверу через основной интерфейс
    // Command::new("ip")
    //     .args(["route", "add", server_ip, "via", "0.0.0.0", "dev", "eth0"])
    //     .status()
    //     .await?;
    //
    // // 2. Весь остальной трафик направляем через TUN
    // Command::new("ip")
    //     .args(["route", "add", "0.0.0.0/1", "dev", tun_name])
    //     .status()
    //     .await?;
    //
    // Command::new("ip")
    //     .args(["route", "add", "128.0.0.0/1", "dev", tun_name])
    //     .status()
    //     .await?;

    Ok(())
}

async fn cleanup_routing(tun_name: &str, server_ip: &str) -> error::Result<()> {
    // Удаляем добавленные маршруты при завершении
    let _ = Command::new("ip")
        .args(["route", "del", server_ip])
        .status()
        .await?;

    let _ = Command::new("ip")
        .args(["route", "del", "0.0.0.0/1", "dev", tun_name])
        .status()
        .await?;

    let _ = Command::new("ip")
        .args(["route", "del", "128.0.0.0/1", "dev", tun_name])
        .status()
        .await?;

    Ok(())
}
