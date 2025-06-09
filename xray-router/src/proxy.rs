use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tokio_socks::tcp::Socks5Stream;

pub async fn send_via_socks5(
    packet: &[u8],
    proxy_addr: &str,
    target_addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = Socks5Stream::connect(proxy_addr, target_addr).await?;
    stream.write_all(packet).await?;
    Ok(())
}
