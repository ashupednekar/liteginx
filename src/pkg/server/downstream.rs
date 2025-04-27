use async_trait::async_trait;
use tokio::{io::{split, AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream}};
use crate::{pkg::spec::routes::{Route, UpstreamTarget}, prelude::{ProxyError, Result}};
use rand::seq::IndexedRandom;

#[async_trait]
pub trait ListenDownstream{
    async fn serve(&self) -> Result<()>;
    async fn handle(&self, conn: &'async_trait mut DownStreamConn) -> Result<()>;
}

pub struct DownStreamConn<'a>{
    pub target: UpstreamTarget,
    pub stream: &'a mut TcpStream
}

impl<'a> DownStreamConn<'a>{
    pub fn new(stream: &'a mut TcpStream, targets: Vec<UpstreamTarget>) -> Result<Self> {
        let target = match targets.choose(&mut rand::rng()) {
            Some(t) => t.clone(),
            None => {
                return Err(ProxyError::DownStreamServerEmptyTargets);
            }
        };
        Ok(Self { target, stream })
    }
}


#[async_trait]
impl ListenDownstream for Route{
    async fn serve(&self) -> Result<()> {
        let listener = TcpListener::bind(&format!("0.0.0.0:{}", &self.listen)).await?;
        loop{
            let (mut stream, _) = listener.accept().await?;
            let mut conn = DownStreamConn::new(&mut stream, self.targets.clone())?;
            self.handle(&mut conn).await?;
        }
    }

    async fn handle(&self, conn: &'async_trait mut DownStreamConn) -> Result<()>{
        let mut buffer = vec![1;1024];
        let (mut reader, mut writer) = split(&mut conn.stream);
        tokio::select! {
            _ = async{
                loop{
                    let n = reader.read(&mut buffer).await?;
                    if n == 0 {
                        break;
                    }
                    let body = buffer[..n].to_vec();
                    conn.target.tx.send(body)?;
                    tracing::info!("received downstream message from client, sent to upstream target");
                }
                Ok::<(), ProxyError>(())
            } => {},
            _ = async{
                let mut rx = self.tx.subscribe();
                while let Ok(msg) = rx.recv().await{
                    writer.write_all(&msg).await?;
                    tracing::info!("received upstream message from target, sent downstream");
                }
                Ok::<(), ProxyError>(())
            } => {}
        } 
        Ok(())
    }
}
