use async_trait::async_trait;
use matchit::Router;
use tokio::{
    sync::{broadcast::{channel, Receiver, Sender}},
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
    async fn forward(&self, mut proxy_rx: Receiver<Vec<u8>>) -> Result<Receiver<Vec<u8>>> {
        let (tx, rx) = channel::<Vec<u8>>(1);
        while let Ok(msg) = proxy_rx.recv().await{

        }
        Ok(rx)
    }
}
