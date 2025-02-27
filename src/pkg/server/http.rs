use async_trait::async_trait;
use futures::future::join_all;
use matchit::Router;
use tokio::task::JoinHandle;

use crate::{
    pkg::conf::spec::HttpRoute,
    prelude::{IoResult, Result},
};

use super::{ForwardRoutes, HttpRoutes, SpawnServers};

#[async_trait]
impl SpawnServers for HttpRoutes {
    async fn listen(&self) -> Result<()> {
        let mut handles: Vec<JoinHandle<IoResult<()>>> = vec![];
        for (port, route) in self.iter() {
            tracing::debug!("loading http server at port: {}", &port);
            let route = route.clone();
            handles.push(self.spawn_tcp_proxy(*port, route).await);
        }
        join_all(handles).await;
        Ok(())
    }
}

#[async_trait]
impl ForwardRoutes for Router<Vec<HttpRoute>> {
    async fn forward(&self, body: Vec<u8>) -> Result<Vec<u8>> {
        Ok("".as_bytes().to_vec())
    }
}
