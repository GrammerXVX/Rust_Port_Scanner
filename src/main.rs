use std::io::{self};
use std::net::{Ipv4Addr, ToSocketAddrs};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::task;
fn read_port(prompt: &str) -> u16 {
    loop {
        println!("{}", prompt);
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Ошибка чтения ввода");

        match input.trim().parse() {
            Ok(num) if num <= 65534 => return num,
            Ok(_) => eprintln!("Порт должен быть ≤ 65535"),
            Err(_) => eprintln!("Введите корректное число"),
        }
    }
}
fn resolve_to_ipv4(host: &str) -> Option<Ipv4Addr> {
    let addrs = format!("{}:0", host).to_socket_addrs().ok()?;

    addrs
        .filter_map(|addr| match addr {
            std::net::SocketAddr::V4(v4) => Some(*v4.ip()),
            _ => None,
        })
        .next()
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ip_str = String::new();
    println!("Введите Ip: ");
    io::stdin().read_line(&mut ip_str)?;
    let ip = Arc::new(
        resolve_to_ipv4(&ip_str.trim())
            .ok_or("Не удалось разрешить IP")
            .unwrap_or(Ipv4Addr::UNSPECIFIED),
    );
    let start_port = read_port("Введите стартовый порт:");
    let end_port = read_port("Введите конечный порт:");
    let ports: Vec<u16> = (start_port..=end_port).collect();
    let mut handles = vec![];

    let (tx, mut rx) = tokio::sync::mpsc::channel(1000);
    for port in ports {
        let ip = ip.clone();
        let tx = tx.clone();
        let handle = task::spawn(async move {
            let addr = (*ip, port);
            match TcpStream::connect(addr).await {
                Ok(_) => {
                    let _ = tx.send(format!("Порт {} открыт", port)).await;
                }
                Err(e) => {
                    let _ = tx.send(format!("Порт {} закрыт, сообщение: {}", port,e)).await;
                }
            }
        });

        handles.push(handle);
    }
    for handle in handles {
        handle.await?;
    }
    drop(tx);
    println!("\n");
    while let Some(result) = rx.recv().await {
        println!("{}", result);
    }

    Ok(())
}
