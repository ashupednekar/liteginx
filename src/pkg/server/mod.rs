use async_trait::async_trait;
use matchit::Router;
use std::collections::HashMap;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::broadcast::{channel, Receiver},
};

use crate::{
    pkg::conf::spec::{HttpRoute, TcpRoute},
    prelude::{IoResult, ProxyError, Result},
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
pub trait ForwardRoutes {
    async fn forward(&self, proxy_rx: Receiver<Vec<u8>>) -> Result<Receiver<Vec<u8>>>;
}

#[async_trait]
pub trait SpawnServers {
    async fn listen(&self) -> Result<()>;
}

pub async fn spawn_tcp_proxy<T>(port: i32, route: T) -> IoResult<()>
where
    T: ForwardRoutes + Send + Sync + Clone + 'static,
{
    let ln = TcpListener::bind(&format!("0.0.0.0:{}", &port))
        .await
        .unwrap();
    tracing::debug!("starting tcp server at port: {}", &port);
    tokio::select!{
        _ = async move {
            loop {
                let route = route.clone();
                let socket = match ln.accept().await {
                    Ok((socket, _)) => socket,
                    Err(_) => {
                        break;
                    }
                };
                tokio::spawn(async move {
                    if handle_connection(socket, route).await.is_err(){
                        tracing::error!("error handling connection");
                    }
                });
            }
        } => {}
    };
    Ok::<(), std::io::Error>(())
}

pub async fn handle_connection<T>(mut socket: TcpStream, route: T) -> Result<()>
where
    T: ForwardRoutes + Send + Sync + Clone + 'static,
{
    let mut buf = vec![0; 1024];
    let (tx, rx) = channel::<Vec<u8>>(1);
    tokio::select! {
        _ = route.forward(rx) => {},
        _ = tokio::spawn(async move{
            loop {
                let n = socket.read(&mut buf).await?;
                if n == 0 {
                    break;
                }
                let body = buf[..n].to_vec();
                tx.send(body)?;
            }
            Ok::<(), ProxyError>(())
        }) => {}
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_test::traced_test;

    #[tokio::test]
    #[traced_test]
    async fn test_server() -> Result<()> {
        let server = Server::new()?;
        server.start().await;
        Ok(())
    }
}
