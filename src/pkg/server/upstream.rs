use async_trait::async_trait;
use crate::{pkg::spec::routes::UpstreamTarget, prelude::Result};

#[async_trait]
pub trait ListenUpsteram{
    async fn listen(&self) -> Result<()>; 
}

#[async_trait]
impl ListenUpsteram for UpstreamTarget{
    async fn listen(&self) -> Result<()>{
        Ok(())
    }
}
