use thiserror::Error;

pub type Result<T> = core::result::Result<T, ProxyError>;


#[derive(Error, Debug)]
pub enum ProxyError{
    #[error("generic error")]
    Generic
}
