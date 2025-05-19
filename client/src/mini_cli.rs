use std::time::Duration;
use tokio::net::UdpSocket;

#[tokio::test]
async fn main_test() {
    // Адрес сервера, к которому подключаемся
    let server_addr = "79.133.182.111:44444"; // Замените на адрес вашего сервера

    // Создаем UDP сокет
    let socket = UdpSocket::bind("0.0.0.0:0").await.unwrap(); // 0.0.0.0:0 - система выберет свободный порт

    // Сообщение для отправки
    let message = "Hello, UDP Server!";

    loop {
        // Отправляем сообщение на сервер
        socket
            .send_to(message.as_bytes(), server_addr)
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}
