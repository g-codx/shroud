use crate::proxy::send_via_socks5;
use etherparse::{IpNumber, Ipv4HeaderSlice, TcpHeaderSlice, UdpHeaderSlice};
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::io;
use std::net::SocketAddr;
use std::process::Command;
use tun::{AbstractDevice, AsyncDevice, Configuration};

pub async fn run_linux() -> io::Result<()> {
    let mut tun_config = Configuration::default();
    tun_config.tun_name("shroud-tun");
    let tun: AsyncDevice = tun::create_as_async(&tun_config)?;
    println!("TUN интерфейс создан");

    setup_tun().await;
    println!("TUN интерфейс сконфигурирован");

    //https://www.wireguard.com/netns/#routing-network-namespace-integration
    
    let mut buf = [0u8; 65535];

    loop {
        let nbytes = tun.recv(&mut buf).await?;
        let packet = &buf[..nbytes];

        if let Ok(ip_slice) = Ipv4HeaderSlice::from_slice(packet) {
            let dst_ip = ip_slice.destination_addr();
            let target_ip_str = dst_ip.to_string();
            dbg!(target_ip_str);

            // if config.target_domains.contains(&target_ip_str) {
            //     match ip_slice.protocol() {
            //         IpNumber::TCP => {
            //             let tcp_slice = TcpHeaderSlice::from_slice(
            //                 &packet[ip_slice.slice().len()..ip_slice.slice().len() + 20],
            //             )
            //             .unwrap();
            //             let dst_port = tcp_slice.destination_port();
            //             let target_addr = SocketAddr::new(dst_ip.into(), dst_port);
            //             let _ = send_via_socks5(packet, &config.socks5_proxy, target_addr).await;
            //         }
            //         IpNumber::UDP => {
            //             let udp_slice =
            //                 UdpHeaderSlice::from_slice(&packet[ip_slice.slice().len()..]).unwrap();
            //             let dst_port = udp_slice.destination_port();
            //             let target_addr = SocketAddr::new(dst_ip.into(), dst_port);
            //             let _ = send_via_socks5(packet, &config.socks5_proxy, target_addr).await;
            //         }
            //         _ => {}
            //     }
            // } else {
            //     direct_send(packet)?;
            // }
        }
    }
}

async fn setup_tun() {
    run_cmd("sudo ip addr add 10.0.0.1/24 dev shroud-tun")
        .await
        .unwrap();
    run_cmd("sudo ip link set shroud-tun up").await.unwrap();
    run_cmd("sudo ip route add default dev shroud-tun")
        .await
        .unwrap();
}

async fn run_cmd(cmd: &str) -> Result<(), String> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let status = tokio::process::Command::new(parts[0])
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

// fn direct_send(packet: &[u8]) -> io::Result<()> {
//     let socket = Socket::new(Domain::ipv4(), Type::raw(), Some(Protocol::ipv4()))?;
//     socket.set_nonblocking(true)?;
//     let addr = SockAddr::from(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)));
//     socket.send_to(packet, &addr)?;
//     Ok(())
// }
