use std::sync::Arc;

use crate::{
    pkg::{
        server::upstream::ListenUpsteram,
        spec::routes::{Route, UpstreamTarget},
    },
    prelude::{ProxyError, Result},
};
use async_trait::async_trait;
use rand::seq::IndexedRandom;
use tokio::{
    io::{split, AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream}, sync::{broadcast::Sender, oneshot}, task::JoinSet
};

#[async_trait]
pub trait ListenDownstream<'a> {
    async fn serve(&self) -> Result<()>;
    //async fn handle(&self, stream: &'a mut TcpStream, target: &'a UpstreamTarget, quit: oneshot::Receiver<()>) -> Result<()>; 
}

#[allow(unreachable_code)] // for ? prop in tcp loop
#[async_trait]
impl<'a> ListenDownstream<'a> for Route {
    async fn serve(&self) -> Result<()> {
        let ln = TcpListener::bind(format!("0.0.0.0:{}", self.listen)).await?;
        let listener = Arc::new(ln);
        let upstream_set = self.targets.iter().cloned().fold(JoinSet::new(), |mut set, target| {
            let tx = self.tx.clone(); 
            set.spawn(async move {
                target.listen(&tx).await
            });
            set
        }); 
        tokio::select! {
            _ = async {
                loop {
                    let (quit_tx, quit_rx) = oneshot::channel::<()>();
                    let target = self.targets.choose(&mut rand::rng()).ok_or_else(|| {
                        return ProxyError::DownStreamServerEmptyTargets;
                    })?;
                    let tx = Arc::new(self.tx.clone());
                    let listener = Arc::clone(&listener);
                    tokio::spawn(async move{
                        let (mut stream, _) = listener.accept().await?;
                        handle(&mut stream, target, tx, quit_rx).await?;
                        Ok::<(), ProxyError>(())
                    });
                }
                Ok::<(), ProxyError>(())
            } => {
                tracing::warn!("downsteam server ended");
            },
            _ = upstream_set.join_all() => {
                tracing::warn!("upstream clients ended");
            },
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("received ctrl_c interrupt, quitting server")
            }
        }
        Ok(())
    }
        

}


async fn handle<'a>(mut stream: &'a mut TcpStream, target: &'a UpstreamTarget, tx: Arc<Sender<Vec<u8>>>, quit: oneshot::Receiver<()>) -> Result<()> {
    let mut buffer = vec![1; 1024];
    let (mut reader, mut writer) = split(&mut stream);
    tokio::select! {
        r = async{
            loop{
                let n = reader.read(&mut buffer).await?;
                if n == 0 {
                    break;
                }
                let body = buffer[..n].to_vec();
                target.tx.send(body)?;
                tracing::debug!("received downstream message from client, sent to upstream target");
            }
            Ok::<(), ProxyError>(())
        } => {
            tracing::debug!("downstream reader closed: {:?}", &r);
        },
        _ = async{
            let mut rx = tx.subscribe();
            while let Ok(msg) = rx.recv().await{
                writer.write_all(&msg).await?;
                tracing::debug!("received upstream message from target, sent downstream");
            }
            Ok::<(), ProxyError>(())
        } => {
            tracing::debug!("downstream listener closed");
        },
        _ = quit => {
            tracing::info!("closing handler");
        }
    }
    Ok(())
}

