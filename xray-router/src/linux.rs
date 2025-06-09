use crate::proxy::send_via_socks5;
use etherparse::{IpNumber, Ipv4HeaderSlice, TcpHeaderSlice, UdpHeaderSlice};
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::io;
use std::net::SocketAddr;
use tun::{AbstractDevice, AsyncDevice, Configuration};

pub async fn run_linux(config: super::AppConfig) -> io::Result<()> {
    let mut tun_config = Configuration::default();
    tun_config.tun_name("shroud-tun");
    let tun: AsyncDevice = tun::create_as_async(&tun_config)?;
    println!("TUN интерфейс создан: {}", tun.tun_name()?);

    let mut buf = [0u8; 65535];

    loop {
        let nbytes = tun.recv(&mut buf).await?;
        let packet = &buf[..nbytes];
        dbg!(packet);

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

// fn direct_send(packet: &[u8]) -> io::Result<()> {
//     let socket = Socket::new(Domain::ipv4(), Type::raw(), Some(Protocol::ipv4()))?;
//     socket.set_nonblocking(true)?;
//     let addr = SockAddr::from(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)));
//     socket.send_to(packet, &addr)?;
//     Ok(())
// }
