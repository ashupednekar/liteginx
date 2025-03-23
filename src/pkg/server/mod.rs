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
}

impl Server {
    pub async fn start(&self) {
        tracing::info!("starting proxy");
        tokio::select! {
            _ = self.tcp_routes.listen_downstream() => {},
            _ = self.tcp_routes.listen_upstream() => {},
            _ = tokio::signal::ctrl_c() => {}
        }
    }
}

#[async_trait]
pub trait SpawnDownstreamServers {
    async fn listen_downstream(&self) -> Result<()>;
}

#[async_trait]
pub trait SpawnUpstreamClients {
    async fn listen_upstream(&self) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_test::traced_test;

    #[tokio::test]
    #[traced_test]
    async fn test_server() -> Result<()> {
        let server = Server::new().await?;
        server.start().await;
        Ok(())
    }
}
