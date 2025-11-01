use anyhow::Result;
use quinn::ConnectionError;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use bincode::{Decode, Encode};

const SERVER_NAME: &str = "localhost";
const LOCALHOST: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);
const CLIENT_ADDR: SocketAddr = SocketAddr::new(LOCALHOST, 5000);
const SERVER_ADDR: SocketAddr = SocketAddr::new(LOCALHOST, 5001);

mod client;
mod codec;
mod quic;
mod server;

use client::*;
use server::*;

#[derive(Debug, Encode, Decode)]
struct Message {
    id: u32,
    content: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut server_builder = ServerBuilder::new(SERVER_ADDR, SERVER_NAME);

    let mut server = server_builder.bind().await?;

    tokio::spawn(async move {
        if let Err(e) = server.run().await {
            eprintln!("Server error: {}", e);
        }
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    let client = ClientBuilder::new(CLIENT_ADDR)
        .set_error_handler(|error| match error {
            ConnectionError::LocallyClosed => {
                println!("[Client] Connection closed locally");
            }
            _ => {
                println!("[Client] Connection Error {:?}", error);
            }
        })
        .connect(&server_builder)
        .await?;

    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    client
        .send_unreliable_message(&Message {
            id: 1,
            content: "Hello from client".to_string(),
        })
        .await?;

    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    client.close_connection(b"Finished Messaging").await;

    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    Ok(())
}
