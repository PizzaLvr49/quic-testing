use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

const SERVER_NAME: &str = "LAN";
const CLIENT_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 4, 134)), 5000);
const SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 4, 134)), 5001);
const NUM_MESSAGES: u64 = 10_000_000;

mod quic;

use quic::*;

#[tokio::main]
async fn main() {
    tokio::spawn(async move {
        if let Err(e) = server().await {
            eprintln!("Server error: {}", e);
        }
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    if let Err(e) = client().await {
        eprintln!("Client error: {}", e);
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
}

async fn client() -> Result<(), Box<dyn Error>> {
    let mut endpoint = bind_client(CLIENT_ADDR)?;
    endpoint.set_default_client_config(client_config()?);
    let connection = endpoint.connect(SERVER_ADDR, SERVER_NAME)?.await?;

    for _ in 0..NUM_MESSAGES {
        send_unreliable(&connection).await?;
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    connection.close(0u32.into(), b"done");

    Ok(())
}

async fn server() -> Result<(), Box<dyn Error>> {
    let endpoint = bind_server(SERVER_ADDR, server_config()?)?;

    while let Some(conn) = endpoint.accept().await {
        let connection = conn.await?;

        receive_datagram(&connection).await?;
    }
    Ok(())
}

pub async fn send_unreliable(connection: &Connection) -> Result<(), Box<dyn Error>> {
    connection.send_datagram(b"hello".to_vec().into())?;
    Ok(())
}

pub async fn receive_datagram(connection: &Connection) -> Result<(), Box<dyn Error>> {
    loop {
        match connection.read_datagram().await {
            Ok(received_bytes) => {
                let s = String::from_utf8_lossy(&received_bytes);
                println!("Received datagram: {}", s);
            }
            Err(ConnectionError::ApplicationClosed(close)) => {
                println!(
                    "Connection closed by peer: {:?}",
                    String::from_utf8_lossy(&close.reason.to_vec())
                );
                break Ok(());
            }
            Err(e) => {
                eprintln!("Error reading datagram: {}", e);
                return Err(e.into());
            }
        }
    }
}
