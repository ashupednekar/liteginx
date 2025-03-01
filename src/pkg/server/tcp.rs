use std::{
    io::{Read, Write},
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

use super::{proxy::spawn_tcp_server, ForwardRoutes, SpawnServers};

#[async_trait]
impl SpawnServers for TcpRoutes {
    async fn listen(&self) -> Result<()> {
        let mut set = JoinSet::new();
        for (port, route) in self.iter() {
            let port = port.clone();
            let route = route.clone();
            set.spawn(spawn_tcp_server(port, route));
        }
        set.join_all().await;
        Ok(())
    }
}


#[async_trait]
impl ForwardRoutes for Vec<TcpRoute> {
    async fn forward(
        &self, 
        mut client_rx: Receiver<Vec<u8>>, 
        server_tx: Sender<Vec<u8>>
    ) -> Result<()> {
        while let Ok(msg) = client_rx.recv().await{
            if let Some(route) = self.choose(&mut rand::thread_rng()) {
                let mut stream = TcpStream::connect(&format!("{}:{}", &route.target_host, &route.target_port)).unwrap();
                stream.write(&msg).unwrap();
                let mut buf = [0; 128];
                stream.read(&mut buf)?;
                server_tx.send(buf.to_vec())?;
            }
        };
        Ok(())
    }
}
