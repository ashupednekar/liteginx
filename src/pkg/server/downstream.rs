use async_trait::async_trait;
use tokio::net::{TcpListener, TcpStream};
use crate::{pkg::spec::routes::{Route, UpstreamTarget}, prelude::{ProxyError, Result}};
use rand::seq::{IndexedRandom, SliceRandom};

#[async_trait]
pub trait ListenDownstream{
    async fn serve(&self) -> Result<()>;
    async fn handle(&self, stream: TcpStream) -> Result<()>;
}

struct DownstreamConnection<'a>{
    pub target: UpstreamTarget,
    pub stream: &'a TcpStream
}

impl<'a> DownstreamConnection<'a>{
    pub fn new(stream: &'a TcpStream, targets: Vec<UpstreamTarget>) -> Result<Self> {
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
            let (stream, _) = listener.accept().await?;
            let conn = DownstreamConnection::new(&stream, self.targets.clone())?;
            self.handle(stream).await?;
        }
    }

    async fn handle(&self, stream: TcpStream) -> Result<()>{

        Ok(())
    }
}
