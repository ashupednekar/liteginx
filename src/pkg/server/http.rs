use async_trait::async_trait;
use matchit::Router;
use tokio::{
    sync::broadcast::{Receiver, Sender},
    task::JoinSet,
};

use crate::{
    pkg::conf::spec::HttpRoute,
    prelude::Result,
};

use super::{spawn_tcp_proxy, ForwardRoutes, HttpRoutes, SpawnServers};

#[async_trait]
impl SpawnServers for HttpRoutes {
    async fn listen(&self) -> Result<()> {
        let mut set = JoinSet::new();
        for (port, route) in self.iter() {
            tracing::debug!("loading http server at port: {}", &port);
            let port = port.clone();
            let route = route.clone();
            set.spawn(spawn_tcp_proxy(port, route));
        }
        set.join_all().await;
        Ok(())
    }
}

#[async_trait]
impl ForwardRoutes for Router<Vec<HttpRoute>> {
    async fn forward(&self, body_ch: Receiver<Vec<u8>>, res_ch: Sender<Vec<u8>>) -> Result<()> {
        Ok(())
    }
}
