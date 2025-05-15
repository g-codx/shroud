use crate::error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UdpSocket;
use tokio::process::Command;
use tun::AbstractDevice;

pub async fn run() -> error::Result<()> {
    //----------------------------------------------------------------------------------------------

    // Создаём и настраиваем TUN-интерфейс
    let mut config = tun::Configuration::default();
    config.tun_name("tun0");

    let mut dev = tun::create_as_async(&config)?;
    println!("TUN интерфейс создан: {}", dev.tun_name()?);

    // Настройка TUN интерфейса в системе (активация и IP)
    setup_tun_interface().await?;

    //----------------------------------------------------------------------------------------------

    // UDP сервер принимает клиентов
    let socket = UdpSocket::bind("0.0.0.0:3000").await?;
    println!("VPN сервер запущен на 0.0.0.0:3000");

    //----------------------------------------------------------------------------------------------

    let mut client_addr = None;

    let mut buf = [0; 5000];
    let mut tun_buf = [0u8; 3000];

    // Принимаем клиентов и обслуживаем их
    loop {
        tokio::select! {
            result = socket.recv_from(&mut buf) => {
                let (size, addr) = result?;
                let raw_packet = &buf[..size];
                let packet = protocol::decrypt(raw_packet)?;
                client_addr = Some(addr);

                println!("Получен и расшифрован пакет от клиента: {}", addr);

                if let Err(e) = dev.write_all(&packet).await {
                    eprintln!("Ошибка записи в TUN: {:?}", e);
                } else {
                    println!("Пакет клиента отправлен в TUN");
                }
            }
            result = dev.read(&mut tun_buf) => {
                let n = result?;

                if n == 0 {
                    println!();
                    return Ok(());
                }

                let packet = &tun_buf[..n];
                let packet = protocol::encrypt(packet)?;

                println!("Получен и зашифрован пакет из TUN");

                if let Some(addr) = client_addr {
                    println!("Адрес клиента известен");
                    socket.send_to(&packet, addr).await?;
                    println!("Пакет отправлен клиенту");
                }
            }
        }
    }
}

// Настройка TUN интерфейса в системе
async fn setup_tun_interface() -> error::Result<()> {
    // Включаем интерфейс
    let output = Command::new("ip")
        .args(["link", "set", "dev", "tun0", "up"])
        .output()
        .await?;

    if !output.status.success() {
        return Err(error::Error::TunConfig(format!(
            "Не удалось поднять tun0: {:?}",
            output
        )));
    }

    // Назначаем IP адресу
    let output = Command::new("ip")
        .args(["addr", "add", "10.8.0.1/24", "dev", "tun0"])
        .output()
        .await?;

    if !output.status.success() {
        return Err(error::Error::TunConfig(format!(
            "Не удалось назначить IP tun0: {:?}",
            output
        )));
    }

    // Включаем форвардинг пакетов (IPv4)
    let output = Command::new("sysctl")
        .args(["-w", "net.ipv4.ip_forward=1"])
        .output()
        .await?;

    if !output.status.success() {
        return Err(error::Error::TunConfig(format!(
            "Не удалось включить ip_forward: {:?}",
            output
        )));
    }

    // Настройка NAT для выхода в интернет (если нужно)
    let output = Command::new("iptables")
        .args([
            "-t",
            "nat",
            "-A",
            "POSTROUTING",
            "-s",
            "10.8.0.0/24",
            "-j",
            "MASQUERADE",
        ])
        .output()
        .await?;

    if !output.status.success() {
        return Err(error::Error::TunConfig(format!(
            "Не удалось настроить NAT: {:?}",
            output
        )));
    }

    println!("TUN интерфейс настроен");
    Ok(())
}
