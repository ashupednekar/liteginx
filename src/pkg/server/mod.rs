use async_trait::async_trait;
use matchit::Router;
use std::{collections::HashMap, sync::mpsc::SendError};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::broadcast::{channel, Sender, Receiver},
    task::JoinHandle,
};

use crate::{
    pkg::conf::spec::{HttpRoute, TcpRoute},
    prelude::{map_ioerr, IoResult, Result},
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
    async fn forward(
        &self, 
        body_ch: Receiver<Vec<u8>>, 
        res_ch: Sender<Vec<u8>>
    ) ->Result<()>;
}

#[async_trait]
pub trait SpawnServers {
    async fn listen(&self) -> Result<()>;
}

pub async fn spawn_tcp_proxy<T>(port: i32, route: T) -> JoinHandle<IoResult<()>>
where
    T: ForwardRoutes + Send + Sync + Clone + 'static,
{
    let ln = TcpListener::bind(&format!("0.0.0.0:{}", &port))
        .await
        .unwrap();
    tokio::spawn(async move {
        loop {
            let route = route.clone();
            let mut socket = match ln.accept().await {
                Ok((socket, _)) => socket,
                Err(_) => {
                    break;
                }
            };
            tokio::spawn(async move {
                let mut buf = vec![0; 1024];
                loop {
                    let (tx, rx) = channel::<Vec<u8>>(1);
                    let n = socket.read(&mut buf).await?;
                    if n == 0 {
                        break;
                    }
                    let body = buf[..n].to_vec();
                    tx.send(body).map_err(map_ioerr)?;
                    route.forward(rx, tx).await.map_err(map_ioerr)?;
                    //route.forward(body_ch, res_ch)
                    /*let res = route.forward(body).await.map_err(|e| {
                        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
                    })?;
                    socket.write_all(&res).await?;*/
                }
                Ok::<(), std::io::Error>(())
            });
        }
        Ok::<(), std::io::Error>(())
    })
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
