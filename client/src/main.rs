mod config;
mod error;

use crate::config::ClientConfig;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UdpSocket;
use tokio::process::Command;
use tun::{AbstractDevice, AsyncDevice, Configuration};
//cargo build --bin client --release && sudo ./target/release/client
//cargo build --release && sudo ./target/release/client

const TUN_NAME: &str = "shroud-tun";

#[tokio::main]
async fn main() -> error::Result<()> {
    let cfg = ClientConfig::load()?;
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    let server_addr = format!("{}:{}", cfg.server_ip, cfg.server_port);

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

    setup_routing(cfg.server_ip.as_str())
        .await
        .expect("Could not setup routing");
    println!("Route настроен: {}", tun.tun_name()?);

    //Обеспечиваем очистку маршрутов при завершении
    let cleanup = cleanup_routing(cfg.server_ip.clone());
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        cleanup.await;
        std::process::exit(0);
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
        }
    }

    Ok(())
}

async fn cleanup_routing(server_ip: String) {
    let route_output = Command::new("ip")
        .arg("route")
        .arg("del")
        .arg(server_ip)
        .output()
        .await
        .expect("Failed to execute IP ROUTE command");

    if !route_output.status.success() {
        eprintln!(
            "Failed to set route: {}",
            String::from_utf8_lossy(&route_output.stderr)
        );
    }
}

async fn setup_routing(server_ip: &str) -> error::Result<()> {
    let interface = get_default_interface().await;
    let gateway = get_gateway().await;

    let route_output = Command::new("ip")
        .arg("route")
        .arg("add")
        .arg("0.0.0.0/0")
        .arg("via")
        .arg("10.0.0.1")
        .arg("dev")
        .arg(TUN_NAME)
        .output()
        .await
        .expect("Failed to execute IP ROUTE command");

    if !route_output.status.success() {
        eprintln!(
            "Failed to set route: {}",
            String::from_utf8_lossy(&route_output.stderr)
        );
    }

    let cidr = format!("{}/32", server_ip);
    Command::new("ip")
        .args([
            "route",
            "add",
            &cidr,
            "via",
            gateway.as_str(),
            "dev",
            interface.as_str(),
        ])
        .status()
        .await
        .expect("Failed to add route to server");

    println!(
        "Route to server added: {} via {} dev {}",
        server_ip, gateway, interface
    );

    Ok(())
}

//ip route show default | awk '{print $5}'
async fn get_default_interface() -> String {
    let output = Command::new("ip")
        .args(["route", "show", "default"])
        .output()
        .await
        .expect("Failed to execute 'ip route' command");

    if output.status.success() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        let interface = output_str.split_whitespace().nth(4).unwrap_or("unknown");
        println!("Default interface: {}", interface);
        interface.to_string()
    } else {
        panic!("Error: Could not determine default interface");
    }
}

//ip route show default | awk '{print $3}'
async fn get_gateway() -> String {
    let output = Command::new("ip")
        .args(["route", "show", "default"])
        .output()
        .await
        .expect("Failed to execute 'ip route' command");

    if output.status.success() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        let interface = output_str.split_whitespace().nth(2).unwrap_or("unknown");
        println!("Gateway: {}", interface);
        interface.to_string()
    } else {
        panic!("Error: Could not determine default interface");
    }
}
