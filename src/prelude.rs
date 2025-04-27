use thiserror::Error;
use tokio::sync::broadcast;

pub type Result<T> = core::result::Result<T, ProxyError>;

#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("generic error")]
    Generic,
    #[error("empty targets, cannot start downstream server")]
    DownStreamServerEmptyTargets,
    #[error("error connecting to upstream target")]
    UpstreamConnectionRefused,
    #[error("error sending message downstream")]
    DownstreamMessageError,
    #[error("io error")]
    IoError(#[from] std::io::Error),
    #[error("error writing to channel")]
    ChannelWriteError(#[from] broadcast::error::SendError<Vec<u8>>)
}
