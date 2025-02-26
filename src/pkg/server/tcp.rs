use ::futures::future::join_all;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream}, sync::futures, task::JoinHandle};

use async_trait::async_trait;
use crate::{pkg::{conf::spec::TcpRoute, server::TcpRoutes}, prelude::{Result, IoResult}};

#[async_trait]
pub trait TcpServer{
    async fn listen(&self) -> Result<()>;
    async fn forward(&self, body: Vec<u8>) -> IoResult<Vec<u8>>;

    async fn spawn_tcp_proxy(&self, port: i32, forward: Vec<TcpRoute>) -> JoinHandle<IoResult<()>>{
        let ln = TcpListener::bind(&format!("0.0.0.0:{}", &port)).await.unwrap();
        tokio::spawn(async move{
            loop{
                let mut socket = match ln.accept().await{
                    Ok((socket, _)) => {socket},
                    Err(_) => {break;}
                };
                tokio::spawn(async move{
                    let mut buf = vec![0; 1024];
                    loop {
                        let n = socket.read(&mut buf).await?;
                        if n == 0{
                            break;
                        }
                        let body = buf[..n].to_vec();
                        //self.forward(body).await;
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



#[async_trait]
impl TcpServer for TcpRoutes{
    async fn listen(&self) -> Result<()>{
        let mut handles: Vec<JoinHandle<IoResult<()>>> = vec![];
        for (port, route) in self.into_iter(){
            handles.push(self.spawn_tcp_proxy(*port, route.to_vec()).await);
        }
        join_all(handles).await;
        Ok(())
    }

    async fn forward(&self, body: Vec<u8>) -> IoResult<Vec<u8>>{
        Ok("".as_bytes().to_vec())
    }
}



