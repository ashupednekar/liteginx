use crate::{
    pkg::{
        conf::settings,
        server::upstream::ListenUpstream,
        spec::routes::{Route, UpstreamTarget},
    },
    prelude::{ProxyError, Result},
};
use async_trait::async_trait;
use humantime::parse_duration;
use rand::seq::IndexedRandom;
use tokio::{
    io::{split, AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{broadcast::Sender, oneshot},
    task::JoinSet,
};

#[async_trait]
pub trait ListenDownstream<'a> {
    async fn serve(&self) -> Result<()>;
    async fn retry(&self) -> Result<()>;
    async fn spawn_upstream(&self) -> Result<JoinSet<Result<()>>>;
}

#[allow(unreachable_code)] // for ? prop in tcp loop
#[async_trait]
impl<'a> ListenDownstream<'a> for Route {
    async fn spawn_upstream(&self) -> Result<JoinSet<Result<()>>> {
        let set = self
            .targets
            .iter()
            .cloned()
            .fold(JoinSet::new(), |mut set, target| {
                let tx = self.tx.clone();
                set.spawn(async move { target.listen(&tx, 0).await });
                set
            });
        Ok(set)
    }

    async fn retry(&self) -> Result<()> {
        tokio::time::sleep(parse_duration(
            &settings
                .upstream_reconnect_heartbeat
                .clone()
                .unwrap_or("10s".into()),
        )?)
        .await;
        self.serve().await?;
        Ok(())
    }

    async fn serve(&self) -> Result<()> {
        if let Err(e) = async{
            let listener = TcpListener::bind(format!("0.0.0.0:{}", self.listen)).await?;
            tracing::debug!("bound to port: {}", &self.listen);
            tokio::select! {
                _ = async {
                    loop {
                        let (_quit_tx, quit_rx) = oneshot::channel::<()>();
                        let (mut stream, _) = listener.accept().await?;
                        let target = self.targets.choose(&mut rand::rng()).ok_or_else(|| {
                            return ProxyError::DownStreamServerEmptyTargets;
                        })?;
                        let target = target.clone();
                        let tx = self.tx.clone();
                        tokio::spawn(async move{
                            handle(&mut stream, &target, &tx, quit_rx).await
                        });
                    }
                    Err::<(), ProxyError>(ProxyError::DownStreamServerEnded)
                } => {
                    tracing::warn!("downsteam server ended");
                },
                _ = async {
                    self.spawn_upstream().await?.join_all().await;
                    Err::<(), ProxyError>(ProxyError::UpstreamClientsEnded)
                } => {},
                _ = tokio::signal::ctrl_c() => {
                    tracing::info!("received ctrl_c interrupt, quitting server")
                }
            }
            Ok::<(), ProxyError>(())
        }.await{
            tracing::error!("{:?}", &e);
            self.retry().await?;
        }
        Ok(())
    }
}

async fn handle<'a>(
    mut stream: &'a mut TcpStream,
    target: &'a UpstreamTarget,
    tx: &'a Sender<Vec<u8>>,
    quit: oneshot::Receiver<()>,
) -> Result<()> {
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
            Err::<(), ProxyError>(ProxyError::DownStreamEndOfBytes)
        } => {
            tracing::debug!("downstream reader closed: {:?}", &r);
        },
        _ = async{
            let mut rx = tx.subscribe();
            while let Ok(msg) = rx.recv().await{
                writer.write_all(&msg).await?;
                tracing::debug!("received upstream message from target, sent downstream");
            }
            Err::<(), ProxyError>(ProxyError::UpStreamEndOfBytes)
        } => {
            tracing::debug!("downstream listener closed");
        },
        //_ = quit => {
        //    tracing::info!("closing handler");
        //}
    }
    Ok(())
}
