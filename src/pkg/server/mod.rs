use async_trait::async_trait;
use matchit::Router;
use std::collections::HashMap;
use tokio::sync::broadcast::{Receiver, Sender};

use crate::{
    pkg::conf::spec::{HttpRoute, TcpRoute},
    prelude::Result,
};

mod http;
mod loader;
mod proxy;
mod tcp;

pub type TcpRoutes = HashMap<i32, Vec<TcpRoute>>;
pub type HttpRoutes = HashMap<i32, Router<Vec<HttpRoute>>>;


#[derive(Debug)]
pub struct Server {
    tcp_routes: TcpRoutes,
    http_routes: HttpRoutes,
    upstream_chans: HashMap<i32, (Sender<Vec<u8>>, Receiver<Vec<u8>>)>
}

impl Server {
    pub async fn start(&self) {
        tracing::info!("starting proxy");
        tokio::select! {
            _ = self.tcp_routes.spawn() => {},
            _ = self.http_routes.spawn() => {},
            _ = tokio::signal::ctrl_c() => {}
        }
    }

    pub async fn preheat_upstream(&mut self){

    }
}

#[async_trait]
pub trait ForwardRoutes {
    async fn forward(&self, client_rx: Receiver<Vec<u8>>, server_tx: Sender<Vec<u8>>)
        -> Result<()>;
}

#[async_trait]
pub trait SpawnServers {
    async fn spawn(&self) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_test::traced_test;

    #[tokio::test]
    #[traced_test]
    async fn test_server() -> Result<()> {
        let server = Server::new()?;
        server.start().await;
        Ok(())
    }
}
