use thiserror::Error;
use tokio::sync::broadcast::error::SendError;

pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;
pub type IoResult<T> = core::result::Result<T, std::io::Error>;

pub fn map_ioerr<E: ToString>(err: E) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, err.to_string())
}


#[derive(Error, Debug)]
pub enum ProxyError{
    #[error("IO error")]
    IOError(#[from] std::io::Error),
    #[error("channel communications error")]
    ChanelCommunicationError(#[from] SendError<Vec<u8>>),
}
