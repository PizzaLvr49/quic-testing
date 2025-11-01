use std::error::Error;
use std::net::SocketAddr;

use quinn::{Connection, ConnectionError, Endpoint};

use crate::{Message, codec::*, quic::*};

pub struct ServerBuilder {
    pub server_addr: SocketAddr,
    pub server_name: &'static str,
}

pub struct ServerHandle {
    connections: Vec<Connection>,
    endpoint: Endpoint,
}

impl ServerBuilder {
    pub fn new(server_addr: SocketAddr, server_name: &'static str) -> Self {
        Self {
            server_addr,
            server_name,
        }
    }

    pub async fn bind(&mut self) -> Result<ServerHandle, Box<dyn Error>> {
        let endpoint = bind_server(self.server_addr, server_config()?)?;

        Ok(ServerHandle {
            connections: Vec::new(),
            endpoint,
        })
    }
}

impl ServerHandle {
    pub async fn run(&mut self) {
        tokio::spawn(self.accept_connections()).await??;
    }

    async fn accept_connections(&mut self) -> Result<(), Box<dyn Error + Send>> {
        while let Some(conn) = self.endpoint.accept().await {
            let connection = conn
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)??;

            self.connections.push(connection);
        }
        Ok(())
    }

    pub async fn receive_datagrams(&self, connection: &Connection) -> Result<(), Box<dyn Error>> {
        loop {
            match connection.read_datagram().await {
                Ok(received_bytes) => {
                    let data: Message = decode(&received_bytes)?;
                    println!("Received datagram: {:#?}", data);
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
}
