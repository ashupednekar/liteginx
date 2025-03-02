use crate::{pkg::conf::spec::HttpRoute, prelude::Result};
use async_trait::async_trait;
use matchit::Router;
use regex::bytes::Regex;
use rand::Rng;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::broadcast::{Receiver, Sender},
    task::JoinSet,
};

use super::{proxy::spawn_tcp_server, ForwardRoutes, HttpRoutes, SpawnServers};

#[async_trait]
impl SpawnServers for HttpRoutes {
    async fn listen(&self) -> Result<()> {
        let mut set = JoinSet::new();
        for (port, route) in self.iter() {
            tracing::debug!("loading http server at port: {}", &port);
            let port = port.clone();
            let route = route.clone();
            set.spawn(spawn_tcp_server(port, route));
        }
        tokio::select! {
            _ = set.join_all() => {},
            _ = tokio::signal::ctrl_c() => {}
        }
        Ok(())
    }
}

fn extract_path(body: &[u8]) -> &str {
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

fn replace_bytes(data: Vec<u8>, search: Vec<u8>, replacement: Vec<u8>) -> Vec<u8>{
    data
        .windows(search.len())
        .enumerate()
        .find(|(_, window)| *window == search)
        .map(|(i, _)| {
            let mut new_data = data.clone();
            new_data.splice(i..i + search.len(), replacement.iter().copied());
            new_data
        })
        .unwrap_or(data)
}

#[async_trait]
impl ForwardRoutes for Router<Vec<HttpRoute>> {
    async fn forward(
        &self,
        mut client_rx: Receiver<Vec<u8>>,
        server_tx: Sender<Vec<u8>>,
    ) -> Result<()> {
        while let Ok(mut msg) = client_rx.recv().await {
            let path = extract_path(&msg);
            tracing::info!("received http message at {}", &path);
            match self.at(&path) {
                Ok(matched) => {
                    let http_routes: Vec<HttpRoute> = matched.value.to_vec();
                    let index = rand::rng().random_range(0..http_routes.len());
                    let route = http_routes[index].clone();
                    tracing::info!("got matching route, routing to {:?}", &route);
                    let mut stream = route.connect().await;
                    if let Some(rewrite) = route.rewrite{
                        let rewrite_key = path.replace(matched.params.get("p").unwrap_or(""), "");
                        tracing::info!("rewriting path: {} to {}", &rewrite_key, &rewrite);
                        let re = Regex::new(&format!("/{}", &rewrite_key))?;
                        msg = re.replace_all(&msg, rewrite.as_bytes().to_vec()).to_vec();
                        //msg = replace_bytes(msg.clone(), path.into(), rewrite.into())
                    }
                    stream.write(&msg).await?;
                    let mut buf = [1; 128];
                    stream.read(&mut buf).await?;
                    server_tx.send(buf.to_vec())?;
                }
                Err(_) => {
                    tracing::warn!("no matching route found, returning 404");
                    server_tx.send("HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n".into())?;
                }
            }
        }
        Ok(())
    }
}
