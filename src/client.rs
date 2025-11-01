use anyhow::Result;
use std::net::SocketAddr;

use quinn::Connection;

use crate::{Message, codec::*, quic::*, server::*};

pub struct ClientBuilder {
    client_addr: SocketAddr,
}

pub struct ClientHandle {
    conn: Connection,
}

impl ClientBuilder {
    pub fn new(client_addr: SocketAddr) -> Self {
        Self { client_addr }
    }

    pub async fn connect(&self, server: &ServerBuilder) -> Result<ClientHandle> {
        let mut endpoint = bind_client(self.client_addr)?;
        endpoint.set_default_client_config(client_config()?);
        let conn = endpoint
            .connect(server.server_addr, server.server_name)?
            .await?;
        println!(
            "Connected to server {:?} ({})",
            server.server_name, server.server_addr
        );
        Ok(ClientHandle { conn })
    }
}

impl ClientHandle {
    pub async fn send_unreliable_message(&self, message: &Message) -> Result<()> {
        let data = encode(message)?;
        self.conn.send_datagram(data.into())?;
        Ok(())
    }
}
