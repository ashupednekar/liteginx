use tokio::net::TcpStream;
use tokio::task::JoinSet;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::pkg::server::SpawnUpstreamClients;
use crate::{
    pkg::server::TcpRoutes,
    prelude::Result,
};
use async_trait::async_trait;

#[async_trait]
impl SpawnUpstreamClients for TcpRoutes {
    async fn listen_upstream(&self) -> Result<()> {
        let mut set = JoinSet::new();
        for (_, routes) in self.iter() {
            let routes = routes.clone();
            for route in routes {
                set.spawn(async move{
                    let destination = format!("{}:{}", &route.target_host, &route.target_port);
                    tracing::debug!("connecting to remote: {}", &destination);
                    match TcpStream::connect(&destination).await{
                        Ok(mut stream) => {
                            //TODO: reconnection and idle issues
                            tracing::info!("✅ Connected to upstream: {:?}", &route);
                            let mut buffer = vec![0; 1024];
                            let (mut recv, mut send) = stream.split();
                            tokio::select! {
                                _ = async{
                                    loop {
                                        match recv.read(&mut buffer).await {
                                            Ok(0) => break,
                                            Ok(n) => {
                                                if let Err(e) = route.upstream_tx.send(buffer[..n].to_vec()){
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
                                    let mut proxy_rx = route.proxy_tx.subscribe();
                                    while let Ok(msg) = proxy_rx.recv().await{
                                         if let Err(e) = send.write_all(&msg).await{
                                            eprintln!("error sending msg to stream: {}", e);
                                            break;               
                                         };
                                    }
                                } => {},
                                _ = tokio::signal::ctrl_c() => {}
                            }
                        },
                        Err(_) => {} 
                    };
                });
            }
        }
        tokio::select! {
            _ = set.join_all() => {},
            _ = tokio::signal::ctrl_c() => {}
        }
        Ok(())
    }
}


