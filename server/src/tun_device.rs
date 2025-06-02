use crate::error;
use log::{error, info};
use std::ops::{Deref, DerefMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio_tun::TunBuilder;

pub struct TunDevice(tokio_tun::Tun);

impl TunDevice {
    pub async fn new() -> error::Result<Self> {
        let tun_name = "tun0";

        // Получаем вектор TUN устройств
        let tun_devices = TunBuilder::new()
            .name(tun_name)
            .address("10.0.0.1".parse().unwrap())
            .netmask("255.255.255.0".parse().unwrap())
            .destination("10.0.0.2".parse().unwrap())
            .up()
            .build()?;

        // setup_tun_interface().await?;

        setup_server_network()
            .await
            .expect("Error setting up server network");

        // Берем первое устройство из вектора
        let tun = tun_devices
            .into_iter()
            .next()
            .ok_or(error::Error::TunConfig(String::from("No TUN device")))?;

        info!("TUN {} interface created and configured", tun.name());

        Ok(Self(tun))
    }

    pub async fn read_packet(&mut self, buf: &mut [u8]) -> error::Result<usize> {
        Ok(self.read(buf).await?)
    }

    pub async fn write_packet(&mut self, packet: &[u8]) -> error::Result<()> {
        Ok(self.write_all(packet).await?)
    }
}

impl Deref for TunDevice {
    type Target = tokio_tun::Tun;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TunDevice {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

async fn setup_server_network() -> Result<(), String> {
    // run_cmd("ip route del default").await?;
    // match run_cmd("ip route add default dev tun0").await {
    //     Ok(_) => {}
    //     Err(err) => {
    //         error!("{}", err);
    //     }
    // }
    //ip route add default via 10.8.0.1 dev tun0
    run_cmd("route add 0.0.0.0/0 via 10.8.0.1 dev tun0").await?;
    run_cmd("sysctl -w net.ipv4.ip_forward=1").await?;
    run_cmd("iptables -t nat -A POSTROUTING -o eth0 -j MASQUERADE").await?;
    run_cmd("iptables -A FORWARD -i tun0 -o eth0 -j ACCEPT").await?;
    Ok(())
}

async fn run_cmd(cmd: &str) -> Result<(), String> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let status = Command::new(parts[0])
        .args(&parts[1..])
        .status()
        .await
        .map_err(|e| format!("Failed to execute {}: {}", cmd, e))?;

    if !status.success() {
        return Err(format!(
            "Command `{}` failed with exit code {:?}",
            cmd,
            status.code()
        ));
    }

    Ok(())
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

    //Назначаем IP адресу
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

    Ok(())
}
