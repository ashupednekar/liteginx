use rand::Rng;
use tokio::net::TcpStream;
use tokio::sync::broadcast;
use tokio::task::JoinSet;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::broadcast::{Receiver, Sender},
};

use crate::{
    pkg::{conf::spec::TcpRoute, server::TcpRoutes},
    prelude::Result,
};
use async_trait::async_trait;

use super::{proxy::spawn_tcp_server, ForwardRoutes, SpawnServers};


impl TcpRoute {
    pub async fn connect(&self) -> (Sender<Vec<u8>>, Receiver<Vec<u8>>) {
        let destination = format!("{}:{}", &self.target_host, &self.target_port);
        tracing::debug!("connecting to remote: {}", &destination);
        let conn = TcpStream::connect(&destination).await.unwrap();
        let (tx, rx) = broadcast::channel::<Vec<u8>>(1);
        tracing::info!("✅ Connected to upstream: {:?}", &self);
        (tx, rx)
    }
}

#[async_trait]
impl SpawnServers for TcpRoutes {
    async fn listen(&self) -> Result<()> {
        let mut set = JoinSet::new();
        for (port, route) in self.iter() {
            let port = port.clone();
            let route = route.clone();
            set.spawn(spawn_tcp_server(port, route));
        }
        tokio::select! {
            _ = set.join_all() => {},
            _ = tokio::signal::ctrl_c() => {}
        }
        Ok(())
    }
}

#[async_trait]
impl ForwardRoutes for Vec<TcpRoute> {
    async fn forward(
        &self,
        mut client_rx: Receiver<Vec<u8>>,
        server_tx: Sender<Vec<u8>>,
    ) -> Result<()> {
        tracing::debug!("forward started");
        while let Ok(msg) = client_rx.recv().await {
            tracing::debug!("received client msg: {:?}", &String::from_utf8(msg.clone()));
            let index = rand::rng().random_range(0..self.len());
            let route = self[index].clone();
            let mut stream = route.connect().await;
            stream.write(&msg).await?;
            tracing::info!("🟡 Reading response from upstream...");
            let mut buf = [0; 128];
            stream.read(&mut buf).await?;
            server_tx.send(buf.to_vec())?;
        }
        Ok(())
    }
}
