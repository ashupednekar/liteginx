use std::{io::Write, net::TcpStream};

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
        for (port, route) in self.iter() {
            tracing::debug!("loading tcp server at port: {}", &port);
            let route = route.clone();
            handles.push(self.spawn_tcp_proxy(*port, route).await);
        }
        join_all(handles).await;
        Ok(())
    }
}


#[async_trait]
impl ForwardRoutes for Vec<TcpRoute>{
    async fn forward(&self, body: Vec<u8>) -> Result<()>{
        tracing::debug!("v: {:?}", &self);
        for route in self.iter(){
            let mut stream = TcpStream::connect(
                &format!("{}:{}", &route.target_host, &route.target_port)
            )?;
            stream.write(&body)?;
        }
        Ok(())
    }
}



