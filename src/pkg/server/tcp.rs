use rand::Rng;
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

use super::SpawnUpstreamClients;
use super::{proxy::spawn_tcp_server, ForwardRoutes, SpawnDownstreamServers};


#[async_trait]
impl SpawnDownstreamServers for TcpRoutes {
    async fn listen_downstream(&self) -> Result<()> {
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
impl SpawnUpstreamClients for TcpRoutes {
    async fn listen_upstream(&self) -> Result<()> {
        tokio::select! {
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
        Ok(())
    }
}
