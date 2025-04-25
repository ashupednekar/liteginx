use serde::Deserialize;


#[derive(Debug, Deserialize)]
pub struct Endpoint{
    pub path: String,
    pub rewrite: Option<String>
}


#[derive(Deserialize, Debug, Clone)]
pub struct UpstreamTarget{
    pub host: String,
    pub port: u16
}


#[derive(Debug, Deserialize)]
pub struct Route{
    pub listen: u16,
    pub endpoints: Vec<Endpoint>,
    pub targets: Vec<UpstreamTarget>
}


