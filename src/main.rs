use std::{
    error::Error,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

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
async fn main() -> Result<(), Box<dyn Error>> {
    let mut server = ServerBuilder::new(SERVER_ADDR, SERVER_NAME);
    let client = ClientBuilder::new(CLIENT_ADDR).connect(&server).await?;

    let server = server.bind().await?;

    tokio::spawn(server.run());

    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    client
        .send_unreliable_message(&Message {
            id: 1,
            content: "Hello from client".to_string(),
        })
        .await?;

    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    Ok(())
}
