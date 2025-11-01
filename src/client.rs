use anyhow::Result;
use std::net::SocketAddr;

use quinn::{Connection, ConnectionError, VarInt};

use crate::{Message, codec::*, quic::*, server::*};

pub struct ClientBuilder {
    client_addr: SocketAddr,
    error_handler: Option<fn(ConnectionError)>,
}

pub struct ClientHandle {
    conn: Connection,
}

impl ClientBuilder {
    pub fn new(client_addr: SocketAddr) -> Self {
        Self {
            client_addr,
            error_handler: None,
        }
    }

    pub fn set_error_handler(mut self, handler: fn(ConnectionError)) -> Self {
        self.error_handler = Some(handler);
        self
    }

    pub async fn connect(self, server: &ServerBuilder) -> Result<ClientHandle> {
        let mut endpoint = bind_client(self.client_addr)?;
        endpoint.set_default_client_config(client_config()?);
        let conn = endpoint
            .connect(server.server_addr, server.server_name)?
            .await?;
        println!(
            "[Client] Connected to server {:?} ({})",
            server.server_name, server.server_addr
        );

        if let Some(handler) = self.error_handler {
            let conn_clone = conn.clone();
            tokio::spawn(async move {
                handler(conn_clone.closed().await);
            });
        }

        Ok(ClientHandle { conn })
    }
}

impl ClientHandle {
    pub async fn send_unreliable_message(&self, message: &Message) -> Result<()> {
        let data = encode(message)?;
        self.conn.send_datagram(data.into())?;
        Ok(())
    }

    pub async fn close_connection(&self, reason: &[u8]) {
        self.conn.close(VarInt::from_u32(0), reason);
    }
}
