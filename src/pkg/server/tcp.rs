use rand::Rng;
use tokio::net::TcpStream;
use tokio::sync::broadcast;
use tokio::task::JoinSet;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::broadcast::{Receiver, Sender},
};

use crate::{
    pkg::{conf::spec::TcpRoute, server::TcpRoutes},
    prelude::Result,
};
use async_trait::async_trait;

use super::{proxy::spawn_tcp_server, ForwardRoutes, SpawnServers};


impl TcpRoute {
    pub async fn listen(&self, mut proxy_rx: Receiver<Vec<u8>>, upstream_tx: Sender<Vec<u8>>) {
        let destination = format!("{}:{}", &self.target_host, &self.target_port);
        tracing::debug!("connecting to remote: {}", &destination);
        let mut stream = TcpStream::connect(&destination).await.unwrap();
        tracing::info!("✅ Connected to upstream: {:?}", &self);
        let mut buffer = vec![0; 1024];
        let (mut recv, mut send) = stream.split();
        //TODO: messages are being mirrored back, fix it
        tokio::select! {
            _ = async{
                loop {
                    match recv.read(&mut buffer).await {
                        Ok(0) => break,
                        Ok(n) => {
                            if let Err(e) = upstream_tx.send(buffer[..n].to_vec()){
                                tracing::error!("error sending msg: {}", e.to_string());
                                break;
                            }
                        }
                        Err(e) => {
                            eprintln!("Error reading from stream: {}", e);
                            break;
                        }
                    }
                }
            } => {},
            _ = async{
                while let Ok(msg) = proxy_rx.recv().await{
                     if let Err(e) = send.write_all(&msg).await{
                        eprintln!("error sending msg to stream: {}", e);
                        break;               
                     };
                }
            } => {},
            _ = tokio::signal::ctrl_c() => {}
        }
    }
}

#[async_trait]
impl SpawnServers for TcpRoutes {
    async fn spawn(&self) -> Result<()> {
        let mut set = JoinSet::new();
        for (port, route) in self.iter() {
            let port = port.clone();
            let route = route.clone();
            set.spawn(spawn_tcp_server(port, route));
        }
        tokio::select! {
            _ = set.join_all() => {},
            _ = tokio::signal::ctrl_c() => {}
        }
        Ok(())
    }
}

#[async_trait]
impl ForwardRoutes for Vec<TcpRoute> {
    async fn forward(
        &self,
        mut client_rx: Receiver<Vec<u8>>,
        server_tx: Sender<Vec<u8>>,
    ) -> Result<()> {
        tracing::debug!("forward started");
        let index = rand::rng().random_range(0..self.len());
        let route = self[index].clone();
        let (proxy_tx, proxy_rx) = broadcast::channel::<Vec<u8>>(1);
        let (upstream_tx, mut upstream_rx) = broadcast::channel::<Vec<u8>>(1);
        tokio::select! {
            _ = route.listen(proxy_rx, upstream_tx) => {},
            _ = async{
                while let Ok(msg) = client_rx.recv().await {
                    if let Err(e) = proxy_tx.send(msg){
                        eprintln!("error sending msg: {}", e);
                        break;
                    }
                }
            } => {},
            _ = async{
                while let Ok(msg) = upstream_rx.recv().await{
                    if let Err(e) = server_tx.send(msg){
                        eprintln!("error sending msg: {}", e);
                        break;
                    }
                }
            } => {},
            _ = tokio::signal::ctrl_c() => {}
        }
        Ok(())
    }
}
