use crate::{
    pkg::{conf::settings, spec::routes::UpstreamTarget},
    prelude::{ProxyError, Result},
};
use async_trait::async_trait;
use humantime::parse_duration;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::broadcast::Sender,
};

#[async_trait]
pub trait ListenUpstream {
    async fn listen(&self, downstream_tx: &Sender<Vec<u8>>, retry_count: u32) -> Result<()>;
    async fn retry(&self, downstream_tx: &Sender<Vec<u8>>, retry_attempt: u32) -> Result<()>;
}

#[async_trait]
impl ListenUpstream for UpstreamTarget {
    async fn retry(&self, downstream_tx: &Sender<Vec<u8>>, mut retry_attempt: u32) -> Result<()> {
        if retry_attempt < settings.upstream_reconnect_max_retries.unwrap_or(10) {
            tokio::time::sleep(parse_duration(
                &settings
                    .upstream_reconnect_heartbeat
                    .clone()
                    .unwrap_or("10s".into()),
            )?)
            .await;
            tracing::info!("reconnecting upstream");
            retry_attempt += 1;
            self.listen(&downstream_tx, retry_attempt).await?;
        }
        //else {
        //    retry_attempt = 0;
        //    self.listen(&downstream_tx, retry_attempt).await?;
        //}
        Ok(())
    }

    async fn listen(&self, downstream_tx: &Sender<Vec<u8>>, retry_attempt: u32) -> Result<()> {
        if let Err(e) = async {
            match TcpStream::connect(&format!("{}:{}", &self.host, &self.port)).await {
                Ok(mut stream) => {
                    tracing::info!("connected to upstream target");
                    let mut buffer = vec![0; 1024];
                    let (mut recv, mut send) = stream.split();
                    tokio::select! {
                        _ = async {
                            loop {
                                match recv.read(&mut buffer).await {
                                    Ok(0) => {
                                        //break
                                        return Err::<(), ProxyError>(ProxyError::UpstreamReaderClosed)
                                    },
                                    Ok(n) => {
                                        if let Err(e) = downstream_tx.send(buffer[..n].to_vec()){
                                            tracing::error!("error sending msg: {}", e.to_string());
                                            //break;
                                            return Err::<(), ProxyError>(ProxyError::UpstreamReaderClosed)
                                        }
                                    }
                                    Err(_e) => {
                                        return Err(ProxyError::DownstreamMessageError);
                                    }
                                }
                            }
                        } => {},
                        _ = async {
                            let mut rx = self.tx.subscribe();
                            while let Ok(msg) = rx.recv().await{
                                send.write_all(&msg).await?;
                            }
                            Err::<(), ProxyError>(ProxyError::UpstreamConnectionClosed)
                        } => {},
                        _ = tokio::signal::ctrl_c() => {}
                    }
                }
                Err(e) => {
                    return Err(ProxyError::UpstreamConnectionRefused(format!("{}", &e)));
                }
            }
            Ok::<(), ProxyError>(())
        }
        .await
        {
            tracing::error!("{:?}", &e);
            self.retry(downstream_tx, retry_attempt).await?;
        };
        Ok(())
    }
}
