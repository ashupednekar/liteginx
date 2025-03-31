use std::collections::HashMap;

use tokio::task::JoinSet;
use crate::pkg::conf::spec::Routes;
use crate::pkg::server::SpawnDownstreamServers;
use crate::{
    pkg::server::Route,
    prelude::{Result, IoResult, ProxyError},
};
use async_trait::async_trait;


use rand::Rng;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};


#[async_trait]
impl SpawnDownstreamServers for HashMap<i32, Routes> {
    async fn listen_downstream(&self) -> Result<()> {
        let mut set = JoinSet::new();
        for (port, routes) in self.iter() {
            tracing::debug!("loading http server at port: {}", &port);
            let port = port.clone();
            let routes = routes.clone();
            set.spawn(spawn_tcp_server(port, routes));
        }
        tokio::select! {
            _ = set.join_all() => {},
            _ = tokio::signal::ctrl_c() => {}
        }
        Ok(())
    }
}

pub async fn spawn_tcp_server(port: i32, routes: Routes) -> IoResult<()> {
    let ln = TcpListener::bind(&format!("0.0.0.0:{}", &port))
        .await
        .unwrap();
    tracing::debug!("starting tcp server at port: {}", &port);
    tokio::select! {
        _ = async move {
            loop {
                let routes = routes.clone();
                let socket = match ln.accept().await {
                    Ok((socket, _)) => socket,
                    Err(_) => {
                        break;
                    }
                };
                tokio::spawn(async move {
                    if handle_connection(socket, routes).await.is_err(){
                        tracing::error!("error handling connection");
                    }
                });
            }
        } => {}
    };
    Ok::<(), std::io::Error>(())
}

pub async fn handle_connection(socket: TcpStream, routes: Routes) -> Result<()> {
    let index = rand::rng().random_range(0..routes.len());
    let route = routes[index].clone(); //TODO: maybe consider stickyness later
    let mut buf = vec![0; 1024];
    let (mut reader, mut writer) = tokio::io::split(socket);
    tokio::select! {
        _ = tokio::spawn(async move{
            loop {
                let n = reader.read(&mut buf).await?;
                if n == 0 {
                    break;
                }
                let body = buf[..n].to_vec();
                tracing::debug!("passing client message: {:?}", String::from_utf8(body.clone()));
                route.proxy_tx.send(body)?;
            }
            Ok::<(), ProxyError>(())
        }) => {},
        _ = tokio::spawn(async move{
            let mut upstream_rx = route.upstream_tx.subscribe();
            while let Ok(msg) = upstream_rx.recv().await{
                writer.write_all(&msg).await?;
            }
            Ok::<(), ProxyError>(())
        }) => {},
        _ = tokio::signal::ctrl_c() => {}
    };
    tracing::info!("connection closed");
    Ok(())
}

