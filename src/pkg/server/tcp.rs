use ::futures::future::join_all;
use tokio::task::JoinHandle;

use crate::{
    pkg::{conf::spec::TcpRoute, server::TcpRoutes},
    prelude::{IoResult, Result},
};
use async_trait::async_trait;

use super::{ForwardRoutes, SpawnServers};



#[async_trait]
impl SpawnServers for TcpRoutes {
    async fn listen(&self) -> Result<()> {
        let mut handles: Vec<JoinHandle<IoResult<()>>> = vec![];
        for (port, route) in self.into_iter() {
            tracing::debug!("loading tcp server at port: {}", &port);
            handles.push(self.spawn_tcp_proxy(*port, route).await);
        }
        join_all(handles).await;
        Ok(())
    }
}


#[async_trait]
impl ForwardRoutes for &Vec<TcpRoute>{
    async fn forward(&self, body: Vec<u8>) -> Result<()>{
        Ok(())
    }
}



