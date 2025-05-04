use matchit::Router;
use serde::Deserialize;
use tokio::sync::mpsc::{Receiver, Sender};

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct Endpoint {
    pub path: String,
    pub rewrite: Option<String>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct UpstreamTarget {
    pub host: String,
    pub port: u16,
}

impl PartialEq for UpstreamTarget {
    fn eq(&self, other: &Self) -> bool {
        self.host == other.host && self.port == other.port
    }
}

#[derive(Debug, Default)]
pub struct Route {
    pub listen: u16,
    pub endpoints: Option<Router<Endpoint>>,
    pub targets: Vec<UpstreamTarget>,
}

pub type SenderCh = Sender<Vec<u8>>;
pub type ReceiverCh = Receiver<Vec<u8>>;

