use thiserror::Error;
use tokio::sync::{broadcast, oneshot};

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
    #[error("upstream connection closed")]
    UpstreamConnectionClosed,
    #[error("upstream reader closed")]
    UpstreamReaderClosed,
    #[error("upstream clients ended")]
    UpstreamClientsEnded,
    #[error("downstream server ended")]
    DownStreamServerEnded,
    #[error("end of bytes received from downstream")]
    DownStreamEndOfBytes,
    #[error("end of bytes received from upstream")]
    UpStreamEndOfBytes,
    #[error("io error")]
    IoError(#[from] std::io::Error),
    #[error("invalid time format error")]
    DurationError(#[from] humantime::DurationError),
    #[error("error writing to channel")]
    ChannelWriteError(#[from] broadcast::error::SendError<Vec<u8>>),
}
