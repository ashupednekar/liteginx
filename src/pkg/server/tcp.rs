use std::{
    io::Write,
    net::TcpStream,
};

use tokio::task::JoinSet;
use rand::seq::SliceRandom;
use tokio::sync::broadcast::{Sender, Receiver};

use crate::{
    pkg::{conf::spec::TcpRoute, server::TcpRoutes},
    prelude::Result,
};
use async_trait::async_trait;

use super::{ForwardRoutes, SpawnServers, spawn_tcp_proxy};

#[async_trait]
impl SpawnServers for TcpRoutes {
    async fn listen(&self) -> Result<()> {
        let mut set = JoinSet::new();
        for (port, route) in self.iter() {
            let port = port.clone();
            let route = route.clone();
            set.spawn(spawn_tcp_proxy(port, route));
        }
        set.join_all().await;
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
        tracing::debug!("routing tcp connection at: {:?}", &self);
        while let Ok(msg) = body_ch.recv().await{
            if let Some(route) = self.choose(&mut rand::thread_rng()) {
                let mut stream = TcpStream::connect(&format!("{}:{}", &route.target_host, &route.target_port)).unwrap();
                stream.write(&msg).unwrap();
                let mut buf = [0; 128];
                stream.write(&mut buf)?;
                res_ch.send(buf.to_vec())?;
            }
        };
        Ok(())
    }
}
