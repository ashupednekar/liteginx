use tokio::task::JoinSet;
use crate::pkg::server::{SpawnDownstreamServers, proxy::spawn_tcp_server};
use crate::{
    pkg::server::TcpRoutes,
    prelude::Result,
};
use async_trait::async_trait;



#[async_trait]
impl SpawnDownstreamServers for TcpRoutes {
    async fn listen_downstream(&self) -> Result<()> {
        let mut set = JoinSet::new();
        for (port, routes) in self.iter() {
            tracing::debug!("loading http server at port: {}", &port);
            let port = port.clone();
            let routes = routes.clone();
            set.spawn(spawn_tcp_server(port, routes));
        }
        tokio::select! {
            _ = set.join_all() => {},
            _ = tokio::signal::ctrl_c() => {}
        }
        Ok(())
    }
}
