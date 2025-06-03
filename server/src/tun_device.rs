use crate::error;
use log::{info};
use std::ops::{Deref, DerefMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio_tun::TunBuilder;

pub struct TunDevice(tokio_tun::Tun);

const TUN_NAME: &str = "shroud-tun";

impl TunDevice {
    pub async fn new() -> error::Result<Self> {
        let tun_devices = TunBuilder::new()
            .name(TUN_NAME)
            .address("10.0.0.1".parse().unwrap())
            .netmask("255.255.255.0".parse().unwrap())
            .destination("10.0.0.2".parse().unwrap())
            .mtu(1500)
            .up()
            .build()?;

        setup_server_network()
            .await
            .expect("Error setting up server network");
        
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
    run_cmd("sysctl -w net.ipv4.ip_forward=1").await?;
    run_cmd("iptables -t nat -A POSTROUTING -s 10.0.0.0/24 -o eth0 -j MASQUERADE").await?;
    run_cmd(format!("iptables -A FORWARD -i {} -o eth0 -j ACCEPT", TUN_NAME).as_str()).await?;
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
