use serde::Deserialize;

use super::routes::UpstreamTarget;

#[derive(Debug, Deserialize)]
pub enum Kind{
    #[serde(alias="http")]
    Http,
    #[serde(alias="tcp")]
    Tcp
}


#[derive(Debug, Deserialize)]
pub struct IngressSpec{
    pub kind: Kind,
    pub path: Option<String>,
    pub listen: u16,
    pub rewrite: Option<String>,
    pub targets: Vec<UpstreamTarget>
}

#[derive(Debug, Deserialize)]
pub struct TlsConf{
    pub enabled: bool
}

#[derive(Debug,  Deserialize)]
pub struct IngressConf{
    pub name: String,
    pub spec: Vec<IngressSpec>,
    pub tls: TlsConf
}


