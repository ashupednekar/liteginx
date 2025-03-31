use async_trait::async_trait;
use std::collections::HashMap;

use crate::{
    pkg::conf::spec::Route,
    prelude::Result,
};

use super::conf::spec::Routes;

mod http;
mod loader;
mod proxy;


#[derive(Debug)]
pub struct Server{
    pub routes: HashMap<i32, Routes>
}

impl Server {
    pub async fn start(&self) {
        tracing::info!("starting proxy");
        tokio::select! {
            _ = self.routes.listen_downstream() => {},
            _ = self.routes.listen_upstream() => {},
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
