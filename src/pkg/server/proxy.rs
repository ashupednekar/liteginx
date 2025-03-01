use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::broadcast::channel,
};

use crate::prelude::{IoResult, ProxyError, Result};

use super::ForwardRoutes;

pub async fn spawn_tcp_server<T>(port: i32, route: T) -> IoResult<()>
where
    T: ForwardRoutes + Send + Sync + Clone + 'static,
{
    let ln = TcpListener::bind(&format!("0.0.0.0:{}", &port))
        .await
        .unwrap();
    tracing::debug!("starting tcp server at port: {}", &port);
    tokio::select! {
        _ = async move {
            loop {
                let route = route.clone();
                let socket = match ln.accept().await {
                    Ok((socket, _)) => socket,
                    Err(_) => {
                        break;
                    }
                };
                tokio::spawn(async move {
                    if handle_connection(socket, route).await.is_err(){
                        tracing::error!("error handling connection");
                    }
                });
            }
        } => {}
    };
    Ok::<(), std::io::Error>(())
}

pub async fn handle_connection<T>(socket: TcpStream, route: T) -> Result<()>
where
    T: ForwardRoutes + Send + Sync + Clone + 'static,
{
    let mut buf = vec![0; 1024];
    let (client_tx, client_rx) = channel::<Vec<u8>>(1);
    let (server_tx, mut server_rx) = channel::<Vec<u8>>(1);
    let (mut reader, mut writer) = tokio::io::split(socket);
    tokio::select! {
        _ = route.forward(client_rx, server_tx) => {},
        _ = tokio::spawn(async move{
            loop {
                let n = reader.read(&mut buf).await?;
                if n == 0 {
                    break;
                }
                let body = buf[..n].to_vec();
                client_tx.send(body)?;
            }
            Ok::<(), ProxyError>(())
        }) => {},
        _ = tokio::spawn(async move{
            while let Ok(msg) = server_rx.recv().await{
                writer.write_all(&msg).await?;
            }
            Ok::<(), ProxyError>(())
        }) => {}
    };
    Ok(())
}
