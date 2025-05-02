use matchit::Router;
use serde::Deserialize;
use tokio::sync::broadcast::{self, Sender};

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct Endpoint {
    pub path: String,
    pub rewrite: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UpstreamTarget {
    pub host: String,
    pub port: u16,
    pub tx: Sender<Vec<u8>>,
}

#[derive(Deserialize)]
struct TargetAddr {
    pub host: String,
    pub port: u16,
}

impl<'de> Deserialize<'de> for UpstreamTarget {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let addr = TargetAddr::deserialize(deserializer)?;
        let (tx, _) = broadcast::channel::<Vec<u8>>(1);
        Ok(Self {
            host: addr.host,
            port: addr.port,
            tx,
        })
    }
}

impl Default for UpstreamTarget {
    fn default() -> Self {
        let (tx, _) = broadcast::channel::<Vec<u8>>(1);
        Self {
            host: String::new(),
            port: 0,
            tx,
        }
    }
}

impl PartialEq for UpstreamTarget {
    fn eq(&self, other: &Self) -> bool {
        self.host == other.host && self.port == other.port
    }
}

#[derive(Debug)]
pub struct Route {
    pub listen: u16,
    pub endpoints: Option<Router<Endpoint>>,
    pub targets: Vec<UpstreamTarget>,
    pub tx: Sender<Vec<u8>>,
}

impl Default for Route {
    fn default() -> Self {
        let (tx, _) = broadcast::channel::<Vec<u8>>(1);
        Route {
            listen: 0,
            endpoints: None,
            targets: vec![],
            tx,
        }
    }
}
