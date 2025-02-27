use async_trait::async_trait;
use matchit::Router;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpListener, task::JoinHandle};
use std::collections::HashMap;

use crate::{
    pkg::conf::spec::{HttpRoute, TcpRoute},
    prelude::{IoResult, Result},
};

mod http;
mod loader;
mod tcp;

pub type TcpRoutes = HashMap<i32, Vec<TcpRoute>>;
pub type HttpRoutes = HashMap<i32, Router<Vec<HttpRoute>>>;

#[derive(Debug)]
pub struct Server {
    tcp_routes: TcpRoutes,
    http_routes: HttpRoutes,
}

impl Server {
    pub async fn start(&self) {
        tracing::info!("starting proxy");
        tokio::select! {
            _ = self.tcp_routes.listen() => {},
            _ = self.http_routes.listen() => {},
            _ = tokio::signal::ctrl_c() => {}
        }
    }
}


#[async_trait]
pub trait ForwardRoutes{
    async fn forward(&self, body: Vec<u8>) -> Result<()>;
}


#[async_trait]
pub trait SpawnServers {
    async fn listen(&self) -> Result<()>;

    async fn spawn_tcp_proxy<T>(&self, port: i32, route: T) -> JoinHandle<IoResult<()>>
    where T: ForwardRoutes + Send + Sync + Clone
    {
        let ln = TcpListener::bind(&format!("0.0.0.0:{}", &port))
            .await
            .unwrap();
        tokio::spawn(async move {
            loop {
                let mut socket = match ln.accept().await {
                    Ok((socket, _)) => socket,
                    Err(_) => {
                        break;
                    }
                };
                tokio::spawn(async move {
                    let mut buf = vec![0; 1024];
                    loop {
                        let n = socket.read(&mut buf).await?;
                        if n == 0 {
                            break;
                        }
                        let body = buf[..n].to_vec();
                        //route.forward(body).await;
                        //send to targets, load-balanced, send response back
                        socket.write_all("OK".as_bytes()).await?;
                    }
                    Ok::<(), std::io::Error>(())
                });
            }
            Ok::<(), std::io::Error>(())
        })
    }
}

#[cfg(test)]
mod tests{
    use tracing_test::traced_test;
    use super::*;
    
    #[tokio::test]
    #[traced_test]
    async fn test_server() -> Result<()>{
        let server = Server::new()?;
        server.start().await;
        Ok(())
    }

}
