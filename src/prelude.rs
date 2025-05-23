use thiserror::Error;
use tokio::sync::mpsc;

pub type Result<T> = core::result::Result<T, ProxyError>;

#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("generic error")]
    Generic,
    #[error("empty targets, cannot start downstream server")]
    DownStreamServerEmptyTargets,
    #[error("error connecting to upstream target")]
    UpstreamConnectionRefused(String),
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
    #[error("json decode error")]
    JSONDecodeError(#[from] serde_json::Error),
    #[error("invalid time format error")]
    DurationError(#[from] humantime::DurationError),
    #[error("error writing to channel")]
    ChannelWriteError(#[from] mpsc::error::SendError<Vec<u8>>),
}
