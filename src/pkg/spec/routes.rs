use serde::{Deserialize, Deserializer};
use tokio::sync::broadcast::{self, Receiver, Sender};


#[derive(Debug, Deserialize)]
pub struct Endpoint{
    pub path: String,
    pub rewrite: Option<String>
}


#[derive(Deserialize, PartialEq, Debug, Clone)]
pub struct UpstreamTarget{
    pub host: String,
    pub port: u16
}


#[derive(Debug)]
pub struct Route {
    pub listen: u16,
    pub endpoints: Vec<Endpoint>,
    pub targets: Vec<UpstreamTarget>,
    pub tx: Sender<Vec<u8>>,
    pub rx: Receiver<Vec<u8>>,
}

impl Default for Route{
    fn default() -> Self {
        let (tx, rx) = broadcast::channel::<Vec<u8>>(1);
        Route { listen: 0, endpoints: vec![], targets: vec![], tx, rx}
    }
}
