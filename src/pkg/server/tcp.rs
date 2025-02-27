use std::{
    io::{Read, Write},
    net::TcpStream,
};

use ::futures::future::join_all;
use rand::seq::SliceRandom;
use tokio::{task::JoinHandle, sync::broadcast::{Sender, Receiver, error::SendError}};

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
impl ForwardRoutes for Vec<TcpRoute> {
    async fn forward(
        &self, 
        mut body_ch: Receiver<Vec<u8>>, 
        res_ch: Sender<Vec<u8>>
    ) -> Result<()>{
        tokio::spawn(async move{
            tracing::debug!("routing tcp connection at: {:?}", &self);
            while let Ok(msg) = body_ch.recv().await{
                if let Some(route) = self.choose(&mut rand::thread_rng()) {
                    let mut stream = TcpStream::connect(&format!("{}:{}", &route.target_host, &route.target_port)).unwrap();
                    stream.write(&msg).unwrap();
                    let mut buf = [0; 128];
                    stream.write(&mut buf);
                    res_ch.send(buf.to_vec())?;
                }
            };
        });
        Ok(())
    }
}
