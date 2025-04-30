use crate::{
    pkg::{conf::settings, spec::routes::UpstreamTarget},
    prelude::{ProxyError, Result},
};
use async_trait::async_trait;
use humantime::parse_duration;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::broadcast::Sender, time::interval,
};

#[async_trait]
pub trait ListenUpsteram {
    async fn listen(&self, downstream_tx: &Sender<Vec<u8>>) -> Result<()>;
}

#[async_trait]
impl ListenUpsteram for UpstreamTarget {
    async fn listen(&self, downstream_tx: &Sender<Vec<u8>>) -> Result<()> {
        //TODO: plan reconnects
        match TcpStream::connect(&format!("{}:{}", &self.host, &self.port)).await {
            Ok(mut stream) => {
                tracing::info!("connected to upstream target");
                let mut buffer = vec![0; 1024];
                let (mut recv, mut send) = stream.split();
                tokio::select! {
                    _ = async {
                        loop {
                            match recv.read(&mut buffer).await {
                                Ok(0) => break,
                                Ok(n) => {
                                    if let Err(e) = downstream_tx.send(buffer[..n].to_vec()){
                                        tracing::error!("error sending msg: {}", e.to_string());
                                        break;
                                    }
                                }
                                Err(_e) => {
                                    return Err(ProxyError::DownstreamMessageError);
                                }
                            }
                        }
                        Ok::<(), ProxyError>(())
                    } => {tracing::warn!("upstream reader closed");},
                    _ = async {
                        let mut rx = self.tx.subscribe();
                        while let Ok(msg) = rx.recv().await{
                            send.write_all(&msg).await?;
                        }
                        Ok::<(), ProxyError>(())
                    } => {tracing::warn!("upstream connection closed");},
                    _ = tokio::signal::ctrl_c() => {}
                }
            }
            Err(_e) => {
                return Err(ProxyError::UpstreamConnectionRefused);
            }
        }
        interval(parse_duration(&settings.upstream_reconnect_heartbeat)?).tick().await;
        tracing::info!("reconnecting upstream");
        self.listen(&downstream_tx).await?;
        Ok(())
    }
}
