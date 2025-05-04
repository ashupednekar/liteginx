use crate::{
    pkg::{
        conf::settings,
        server::{helpers::{extract_path, http_404_response, match_prefix, rewrite_path}, upstream::ListenUpstream},
        spec::routes::{Connection, Endpoint, Route, UpstreamTarget},
    },
    prelude::{ProxyError, Result},
};
use async_trait::async_trait;
use humantime::parse_duration;
use matchit::Router;
use rand::seq::IndexedRandom;
use tokio::{
    io::{split, AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    task::JoinSet,
};

#[async_trait]
pub trait ListenDownstream<'a> {
    async fn serve(&self) -> Result<()>;
    async fn retry(&self) -> Result<()>;
    async fn spawn_upstream(&self, conn: &'a Connection) -> Result<JoinSet<Result<()>>>;
}

#[async_trait]
impl<'a> ListenDownstream<'a> for Route {
    async fn spawn_upstream(&self, 
        conn: &'a Connection,
    ) -> Result<JoinSet<Result<()>>> {
        let mut set = JoinSet::new();
        let target = self.targets.choose(&mut rand::rng()).ok_or(ProxyError::DownStreamServerEmptyTargets)?;
        let conn = conn.clone();
        let target = target.clone();
        set.spawn(async move { target.listen(&conn, 0).await });
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
        if let Err(e) = async {
            let listener = TcpListener::bind(format!("0.0.0.0:{}", self.listen)).await?;
            tracing::debug!("bound to port: {}", &self.listen);
            loop {
                let (mut stream, _) = listener.accept().await?;
                let endpoints = self.endpoints.clone();
                let conn = Connection::new();
                let mut set = self.spawn_upstream(&conn).await?;
                set.spawn(async move {
                    handle(endpoints, &mut stream, &conn).await 
                });
                tokio::spawn(async move{
                    set.join_all().await;
                });
            }
            #[allow(unreachable_code)]
            Ok::<(), ProxyError>(())
        }
        .await
        {
            tracing::error!("serve error: {:?}", e);
            self.retry().await?;
        }
        Ok(())
    }
}

async fn handle<'a>(
    endpoints: Option<Router<Endpoint>>,
    mut stream: &'a mut TcpStream,
    conn: &'a Connection
) -> Result<()> {
    let mut buffer = vec![1; 1024];
    tracing::debug!("handling connection...");
    let (mut reader, mut writer) = split(&mut stream);
    tokio::select! {
        r = async{
            loop{
                let n = reader.read(&mut buffer).await?;
                if n == 0 {
                    break;
                }
                let mut body = buffer[..n].to_vec();
                if let Some(ref router) = endpoints{
                    let path = extract_path(&body);
                    match match_prefix(&router, &path){
                        Some(endpoint) => {
                            if let Some(ref rewrite) = endpoint.rewrite{
                                let rewrite_from = format!("/{}", &path);
                                tracing::info!("rewriting path: {:?} to {:?}", &rewrite_from, &rewrite);
                                body = rewrite_path(&body, rewrite_from.into(), rewrite.as_str().into());
                            }
                            conn.target_tx.send(body)?;
                        },
                        None => {
                            tracing::warn!("path {} not found", &path);
                            conn.client_tx.send(http_404_response()?.into())?;
                        }
                    }
                }else{
                    conn.target_tx.send(body)?;
                }
                tracing::debug!("received downstream message from client, sent to upstream target");
            }
            Err::<(), ProxyError>(ProxyError::DownStreamEndOfBytes)
        } => {
            tracing::debug!("downstream reader closed: {:?}", &r);
        },
        _ = async{
            let mut rx = conn.client_tx.subscribe();
            while let Ok(msg) = rx.recv().await{
                writer.write_all(&msg).await?;
                tracing::debug!("received upstream message from target, sent downstream");
            }
            Err::<(), ProxyError>(ProxyError::UpStreamEndOfBytes)
        } => {
            tracing::debug!("downstream listener closed");
        },
    }
    Ok(())
}
