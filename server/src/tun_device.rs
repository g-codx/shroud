use crate::error;
use log::info;
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
            .up()
            .build()?;

        setup_tun_interface().await?;

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
    
    Ok(())
}
