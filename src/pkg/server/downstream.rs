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
    io::{split, AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::broadcast::Sender,
};

#[async_trait]
pub trait ListenDownstream<'a> {
    async fn serve(&self) -> Result<()>;
    async fn handle(&self, conn: &'a mut DownStreamConn) -> Result<()>;
}

pub struct DownStreamConn<'a> {
    pub target: &'a UpstreamTarget,
    pub stream: &'a mut TcpStream,
}

impl<'a> DownStreamConn<'a> {
    pub async fn new(
        stream: &'a mut TcpStream,
        target: &'a UpstreamTarget,
        tx: &'a Sender<Vec<u8>>,
    ) -> Result<Self> {
        tracing::debug!("new downstream connection");
        tokio::select! {
            _ = async {
                target.listen(tx).await?;
                Ok::<(), ProxyError>(())
            }=> {tracing::warn!("downsream listener stopped");},
            _ = tx.closed() => {tracing::warn!("downstream channel closed");}
        }
        target.listen(tx).await?;
        Ok(Self { target, stream })
    }
}

#[async_trait]
impl<'a> ListenDownstream<'a> for Route {
    async fn serve(&self) -> Result<()> {
        let listener = TcpListener::bind(&format!("0.0.0.0:{}", &self.listen)).await?;
        loop {
            let (mut stream, _) = listener.accept().await?;
            let target = self.targets.choose(&mut rand::rng()).ok_or_else(|| {
                return ProxyError::DownStreamServerEmptyTargets;
            })?;
            let mut conn = DownStreamConn::new(&mut stream, &target, &self.tx).await?;
            self.handle(&mut conn).await?;
        }
    }

    async fn handle(&self, conn: &'a mut DownStreamConn) -> Result<()> {
        let mut buffer = vec![1; 1024];
        let (mut reader, mut writer) = split(&mut conn.stream);
        tokio::select! {
            r = async{
                loop{
                    let n = reader.read(&mut buffer).await?;
                    if n == 0 {
                        break;
                    }
                    let body = buffer[..n].to_vec();
                    tracing::debug!("channel: {:?}", &conn.target.tx);
                    conn.target.tx.send(body)?;
                    tracing::info!("received downstream message from client, sent to upstream target");
                }
                Ok::<(), ProxyError>(())
            } => {
                tracing::debug!("downstream reader closed: {:?}", &r);
            },
            _ = async{
                let mut rx = self.tx.subscribe();
                while let Ok(msg) = rx.recv().await{
                    writer.write_all(&msg).await?;
                    tracing::info!("received upstream message from target, sent downstream");
                }
                Ok::<(), ProxyError>(())
            } => {
                tracing::debug!("downstream listener closed");
            },
            _ = tokio::signal::ctrl_c() => {}
        }
        Ok(())
    }
}
