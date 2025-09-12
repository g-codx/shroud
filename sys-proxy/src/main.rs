use std::net::{SocketAddr, ToSocketAddrs};
use std::str::FromStr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};

const PROXY_PORT: u16 = 1234;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Настраиваем iptables
    setup_iptables(PROXY_PORT)?;

    // Запускаем TCP-прокси
    let tcp_listener = TcpListener::bind(format!("0.0.0.0:{}", PROXY_PORT)).await?;
    println!("TCP-прокси слушает на порту {}", PROXY_PORT);

    // Запускаем UDP-прокси
    let udp_socket = UdpSocket::bind(format!("0.0.0.0:{}", PROXY_PORT)).await?;
    println!("UDP-прокси слушает на порту {}", PROXY_PORT);

    tokio::spawn(async move {
        loop {
            let (mut client_tcp, _) = tcp_listener.accept().await.unwrap();
            tokio::spawn(async move {
                handle_tcp(&mut client_tcp).await;
            });
        }
    });

    tokio::spawn(async move {
        let mut buf = [0u8; 65536];
        loop {
            let (len, addr) = udp_socket.recv_from(&mut buf).await.unwrap();
            handle_udp(&udp_socket, &buf[..len], addr).await;
        }
    });

    // Ожидаем завершения (Ctrl+C)
    tokio::signal::ctrl_c().await?;
    cleanup_iptables()?;
    Ok(())
}

async fn handle_tcp(client: &mut TcpStream) {
    let mut buf = [0u8; 4096];
    let len = match client.read(&mut buf).await {
        Ok(len) => len,
        Err(e) => {
            eprintln!("Ошибка чтения: {}", e);
            return;
        }
    };

    // Динамическое определение адреса
    let target_addr = match extract_target_addr(&buf[..len]) {
        Some(addr) => addr,
        None => {
            eprintln!("Не удалось определить адрес");
            return;
        }
    };

    let mut target = match TcpStream::connect(target_addr).await {
        Ok(stream) => stream,
        Err(e) => {
            eprintln!("Ошибка подключения к {}: {}", target_addr, e);
            return;
        }
    };

    if let Err(e) = target.write_all(&buf[..len]).await {
        eprintln!("Ошибка отправки: {}", e);
    }
}



fn extract_target_addr(data: &[u8]) -> Option<SocketAddr> {
    // Пытаемся распарсить HTTP/HTTPS
    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut req = httparse::Request::new(&mut headers);

    // Парсим только если это похоже на HTTP-запрос
    if !data.starts_with(b"GET ") && !data.starts_with(b"POST ") && !data.starts_with(b"CONNECT ") {
        return None;
    }

    // Парсим заголовки
    let res = req.parse(data).ok()?;
    if res.is_complete() {
        if let Some(host_header) = req.headers.iter().find(|h| h.name.eq_ignore_ascii_case("host")) {
            let host_str = std::str::from_utf8(host_header.value).ok()?;
            dbg!(host_str);

            // Обрабатываем доменное имя с портом
            if let Ok(addr) = resolve_host_port(host_str) {
                dbg!(addr);
                return Some(addr);
            }
        }
    }

    None
}

fn resolve_host_port(host_port: &str) -> Result<SocketAddr, Box<dyn std::error::Error>> {
    // Разделяем хост и порт
    let (host, port) = if let Some(pos) = host_port.find(':') {
        (&host_port[..pos], host_port[pos+1..].parse().unwrap_or(80))
    } else {
        (host_port, 80)  // Порт по умолчанию
    };

    // Разрешаем доменное имя в IP-адрес
    let addr = format!("{}:{}", host, port)
        .to_socket_addrs()?
        .next()
        .ok_or("Failed to resolve address")?;

    Ok(addr)
}

// fn extract_target_addr(data: &[u8]) -> Option<SocketAddr> {
//     let mut headers = [httparse::EMPTY_HEADER; 16];
//     let mut req = httparse::Request::new(&mut headers);
//
//     match req.parse(data) {
//         Ok(_) => {}
//         Err(err) => {
//             eprintln!("{}", err);
//         }
//     }
//
//     if let Some(host) = req.headers.iter().find(|h| h.name.eq_ignore_ascii_case("host")) {
//         let host_str = std::str::from_utf8(host.value).ok()?;
//         dbg!(host_str);
//         return SocketAddr::from_str(host_str).ok();
//     } else {
//         dbg!("{}", &req);
//     }
//
//     "8.8.8.8:53".parse().ok() // Google DNS
// }

async fn handle_udp(socket: &UdpSocket, data: &[u8], addr: SocketAddr) {
    // Анализ UDP-трафика (VoIP, игры)
    println!(
        "Получен UDP-пакет ({} байт) от {}: {:?}",
        data.len(),
        addr,
        data
    );

    // Пример: блокируем UDP-пакеты с определённым содержимым
    if data.starts_with(b"RTP") {
        // RTP = аудио/видео поток
        println!("Обнаружен RTP-пакет (аудио/видео)");
    }

    // Перенаправляем трафик дальше (например, на сервер игры)
    let target_addr = "1.2.3.4:27015".parse::<SocketAddr>().unwrap(); // Пример для CS:GO
    socket.send_to(data, target_addr).await.unwrap();
}

fn setup_iptables(proxy_port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let ipt = iptables::new(false)?;

    // Перенаправляем весь TCP-трафик на наш прокси
    ipt.append(
        "nat",
        "PREROUTING",
        &format!("-p tcp -j REDIRECT --to-port {}", proxy_port),
    )?;

    // Для UDP (например, VoIP, игры)
    ipt.append(
        "nat",
        "PREROUTING",
        &format!("-p udp -j REDIRECT --to-port {}", proxy_port),
    )?;

    Ok(())
}

fn cleanup_iptables() -> Result<(), Box<dyn std::error::Error>> {
    let ipt = iptables::new(false)?;
    ipt.delete("nat", "PREROUTING", "-p tcp -j REDIRECT --to-port 1234")?;
    ipt.delete("nat", "PREROUTING", "-p udp -j REDIRECT --to-port 1234")?;
    Ok(())
}
