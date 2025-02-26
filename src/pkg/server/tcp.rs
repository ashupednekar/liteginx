use ::futures::future::join_all;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    task::JoinHandle,
};

use crate::{
    pkg::{conf::spec::TcpRoute, server::TcpRoutes},
    prelude::{IoResult, Result},
};
use async_trait::async_trait;

use super::ForwardRoutes;



#[async_trait]
impl TcpServer for TcpRoutes {
    async fn listen(&self) -> Result<()> {
        let mut handles: Vec<JoinHandle<IoResult<()>>> = vec![];
        for (port, route) in self.into_iter() {
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



#[async_trait]
pub trait TcpServer {
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
