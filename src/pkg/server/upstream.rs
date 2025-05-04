use crate::{
    pkg::{
        conf::settings,
        spec::routes::{ReceiverCh, SenderCh, UpstreamTarget},
    },
    prelude::{ProxyError, Result},
};
use async_trait::async_trait;
use humantime::parse_duration;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[async_trait]
pub trait ListenUpstream {
    async fn listen(
        &self,
        client_tx: SenderCh,
        target_rx: ReceiverCh,
        retry_attempt: u32,
    ) -> Result<()>;
    async fn retry(
        &self,
        client_tx: SenderCh,
        target_rx: ReceiverCh,
        retry_attempt: u32,
    ) -> Result<()>;
}

#[async_trait]
impl ListenUpstream for UpstreamTarget {
    async fn retry(
        &self,
        client_tx: SenderCh,
        target_rx: ReceiverCh,
        mut retry_attempt: u32,
    ) -> Result<()> {
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
            self.listen(client_tx, target_rx, retry_attempt).await?;
        }
        Ok(())
    }

    async fn listen(
        &self,
        client_tx: SenderCh,
        mut target_rx: ReceiverCh,
        retry_attempt: u32,
    ) -> Result<()> {
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
                                        if let Err(e) = client_tx.send(buffer[..n].to_vec()).await{
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
                            while let Some(msg) = target_rx.recv().await{
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
            self.retry(client_tx, target_rx, retry_attempt).await?;
        };
        Ok(())
    }
}
