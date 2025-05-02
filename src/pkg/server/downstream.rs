use crate::{
    pkg::{
        conf::settings,
        server::upstream::ListenUpstream,
        spec::routes::{Endpoint, Route, UpstreamTarget},
    },
    prelude::{ProxyError, Result},
};
use async_trait::async_trait;
use humantime::parse_duration;
use matchit::Router;
use rand::seq::IndexedRandom;
use serde_json::json;
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
        if let Err(e) = async {
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
                        let endpoints = self.endpoints.clone();
                        let tx = self.tx.clone();
                        tokio::spawn(async move{
                            handle(endpoints, &mut stream, &target, &tx, quit_rx).await
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
        }
        .await
        {
            tracing::error!("{:?}", &e);
            self.retry().await?;
        }
        Ok(())
    }
}

async fn handle<'a>(
    endpoints: Option<Router<Endpoint>>,
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
                            target.tx.send(body)?;
                        },
                        None => {
                            tracing::warn!("path {} not found", &path);
                            tx.send(http_404_response()?.into())?;
                        }
                    }
                }
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

pub fn extract_path(body: &[u8]) -> &str {
    let mut lines = body.split(|&b| b == b'\r' || b == b'\n');
    if let Some(request_line) = lines.next() {
        let mut parts = request_line.splitn(3, |&b| b == b' ');
        parts.next();
        if let Some(uri) = parts.next() {
            let path = std::str::from_utf8(uri).unwrap_or("/");
            return path.strip_prefix('/').unwrap_or(path);
        }
    }
    ""
}

fn match_prefix<'a>(router: &'a Router<Endpoint>, path: &str) -> Option<&'a Endpoint> {
    let mut parts: Vec<&str> = path.trim_end_matches('/').split('/').collect();

    while !parts.is_empty() {
        let try_path = format!("/{}", parts.join("/"));
        if let Ok(m) = router.at(&try_path) {
            return Some(m.value);
        }
        parts.pop();
    }
    None
}

fn rewrite_path(data: &[u8], search: Vec<u8>, replacement: Vec<u8>) -> Vec<u8> {
    data.windows(search.len())
        .enumerate()
        .find(|(_, window)| *window == search)
        .map(|(i, _)| {
            let mut new_data = data.to_vec();
            new_data.splice(i..i + search.len(), replacement.iter().copied());
            new_data
        })
        .unwrap_or_else(|| data.to_vec())
}

pub fn http_404_response() -> Result<String> {
    let body = serde_json::to_string(&json!({
        "detail": &settings.not_found_message.clone().unwrap_or("not found".into())
    }))?;
    let content_length = body.len();
    Ok(format!(
        "HTTP/1.1 404 Not Found\r\n\
        Content-Type: application/json\r\n\
        Content-Length: {}\r\n\
        Connection: close\r\n\
        \r\n\
        {}",
        content_length, body
    ))
}
