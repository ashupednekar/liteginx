use crate::{pkg::conf::spec::{HttpRoute, TcpRoute}, prelude::Result};
use async_trait::async_trait;
use matchit::Router;
use rand::Rng;
use tokio::{
    sync::broadcast::{self, Receiver, Sender}, task::JoinSet
};

use super::{proxy::spawn_tcp_server, ForwardRoutes, HttpRoutes, SpawnServers};

impl HttpRoute {
    pub async fn listen(&self, proxy_rx: Receiver<Vec<u8>>, upstream_tx: Sender<Vec<u8>>){
        TcpRoute{
            target_host: self.target_host.clone(),
            target_port: self.target_port
        }.listen(proxy_rx, upstream_tx).await
    }
}


#[async_trait]
impl SpawnServers for HttpRoutes {
    async fn spawn(&self) -> Result<()> {
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

fn replace_bytes(data: Vec<u8>, search: Vec<u8>, replacement: Vec<u8>) -> Vec<u8> {
    data.windows(search.len())
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
        let (proxy_tx, proxy_rx) = broadcast::channel::<Vec<u8>>(1);
        let (upstream_tx, mut upstream_rx) = broadcast::channel::<Vec<u8>>(1);
        tokio::select! {
            //_ = route.listen(proxy_rx, upstream_tx) => {},
            _ = async{
                while let Ok(mut msg) = client_rx.recv().await {

                    let path = extract_path(&msg);
                    tracing::info!("received http message at {}", &path);
                    match self.at(&path) {
                        Ok(matched) => {
                            let http_routes: Vec<HttpRoute> = matched.value.to_vec();
                            let index = rand::rng().random_range(0..http_routes.len());
                            let route = http_routes[index].clone(); 
                            tracing::info!("got matching route, routing to {:?}", &route);
                            if let Some(rewrite) = route.rewrite {
                                let rewrite_key = path.replace(matched.params.get("p").unwrap_or(""), "");
                                tracing::info!("rewriting path: {} to {}", &rewrite_key, &rewrite);
                                msg = replace_bytes(
                                    msg.clone(),
                                    format!("/{}", &rewrite_key).into(),
                                    rewrite.into(),
                                )
                            }
                            if let Err(e) = proxy_tx.send(msg){
                                eprintln!("error sending msg: {}", e);
                                break;
                            }
                        }
                        Err(_) => {
                            tracing::warn!("no matching route found, returning 404");
                            if let Err(e) = server_tx.send("HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n".into()){
                                eprintln!("error sending msg: {}", e);
                                break;
                            }
                        }
                    }
                }
            } => {},
            _ = async{
                while let Ok(msg) = upstream_rx.recv().await{
                    if let Err(e) = server_tx.send(msg){
                        eprintln!("error sending msg: {}", e);
                        break;
                    }
                }
            } => {},
            _ = tokio::signal::ctrl_c() => {}
        }
        Ok(())
    }
}
