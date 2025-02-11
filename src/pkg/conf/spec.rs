use serde::{Deserialize, Deserializer};
use serde::de::Error;
use serde_json::Value;


#[derive(Deserialize, Clone)]
pub struct HttpRoute{
    pub host: String,
    pub port: i32,
    pub rewrite: Option<String>
}

#[derive(Deserialize, Clone)]
pub struct Http{
    #[serde(default = "default_http_kind")]
    pub kind: String,
    pub path: String,
    pub route: HttpRoute
}

#[derive(Deserialize, Clone)]
pub struct Tcp{
    #[serde(default = "default_tcp_kind")]
    pub kind: String,
    pub port: i32
}

#[derive(Deserialize, Clone)]
pub enum Spec{
    Http(Http),
    Tcp(Tcp)
}

#[derive(Deserialize, Clone)]
pub struct Tls{
    pub enabled: bool
}

#[derive(Deserialize, Clone)]
pub struct Config{
    pub name: String,
    pub spec: Spec,
    pub tls: Tls
}


fn default_http_kind() -> String {
    "http".to_string()
}

fn default_tcp_kind() -> String {
    "tcp".to_string()
}

fn deserialize_spec<'de, D>(deserializer: D) -> Result<Spec, D::Error>
where
    D: Deserializer<'de>,
{
    let v: Value = Deserialize::deserialize(deserializer)?;
    if let Some(kind) = v.get("kind").and_then(|k| k.as_str()) {
        match kind {
            "http" => Ok(Spec::Http(serde_json::from_value(v).map_err(D::Error::custom)?)),
            "tcp" => Ok(Spec::Tcp(serde_json::from_value(v).map_err(D::Error::custom)?)),
            _ => Err(D::Error::custom("Unknown kind")),
        }
    } else {
        Err(D::Error::custom("Missing `kind` field"))
    }
}
