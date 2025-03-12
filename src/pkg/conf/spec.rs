use std::i32;

use serde::de::Error;
use serde::{Deserialize, Deserializer};
use serde_yaml::Value;
#[derive(Deserialize, Debug, Clone)]
pub struct HttpRoute {
    pub host: Option<String>,
    pub target_host: String,
    pub target_port: i32,
    pub rewrite: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Http {
    #[serde(default = "default_http_kind")]
    pub kind: String,
    pub path: String,
    pub listen_port: i32,
    pub routes: Vec<HttpRoute>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TcpRoute {
    pub target_host: String,
    pub target_port: i32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Tcp {
    #[serde(default = "default_tcp_kind")]
    pub kind: String,
    pub listen_port: i32,
    pub routes: Vec<TcpRoute>,
}

#[derive(Deserialize, Debug, Clone)]
pub enum Spec {
    Http(Http),
    Tcp(Tcp),
}

#[derive(Deserialize, Debug, Clone)]
pub struct Tls {
    pub enabled: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub name: String,
    #[serde(deserialize_with = "deserialize_spec")]
    pub spec: Spec,
    pub tls: Tls,
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
            "http" => Ok(Spec::Http(
                serde_yaml::from_value(v).map_err(D::Error::custom)?,
            )),
            "tcp" => Ok(Spec::Tcp(
                serde_yaml::from_value(v).map_err(D::Error::custom)?,
            )),
            _ => Err(D::Error::custom("Unknown kind")),
        }
    } else {
        Err(D::Error::custom("Missing `kind` field"))
    }
}

#[cfg(test)]
mod tests {

    use std::fs;

    use crate::{pkg::conf::spec::*, prelude::Result};

    #[test]
    fn test_normal_http_deserialize() -> Result<()> {
        let conf_yaml = fs::read_to_string("src/pkg/conf/fixtures/liteginx/one.yaml")?;
        let config: Config = serde_yaml::from_str(&conf_yaml)?;
        assert_eq!(config.name, "one-ingress");
        if let Spec::Http(spec) = config.spec {
            assert_eq!(spec.kind, "http");
            assert_eq!(spec.path, "/one");
            assert_eq!(spec.listen_port, 80);
            let route = spec.routes[0].clone();
            assert_eq!(route.target_host, "localhost");
            assert_eq!(route.target_port, 3000);
            assert_eq!(route.rewrite, None);
        } else {
            assert!(true);
        }
        assert_eq!(config.tls.enabled, false);
        Ok(())
    }

    #[test]
    fn test_normal_http_deserialize_with_rewrite() -> Result<()> {
        let conf_yaml = fs::read_to_string("src/pkg/conf/fixtures/liteginx/two.yaml")?;
        let config: Config = serde_yaml::from_str(&conf_yaml)?;
        assert_eq!(config.name, "two-ingress");
        if let Spec::Http(spec) = config.spec {
            assert_eq!(spec.kind, "http");
            assert_eq!(spec.path, "/two");
            assert_eq!(spec.listen_port, 80);
            let route = spec.routes[0].clone();
            assert_eq!(route.target_host, "localhost");
            assert_eq!(route.target_port, 3001);
            assert_eq!(route.rewrite, Some("/".to_string()));
        } else {
            assert!(true);
        }
        assert_eq!(config.tls.enabled, false);
        Ok(())
    }

    #[test]
    fn test_tcp_proxy() -> Result<()> {
        let conf_yaml = fs::read_to_string("src/pkg/conf/fixtures/liteginx/redis.yaml")?;
        let config: Config = serde_yaml::from_str(&conf_yaml)?;
        assert_eq!(config.name, "redis-ingress");
        if let Spec::Tcp(spec) = config.spec {
            assert_eq!(spec.routes[0].target_host, "localhost");
            assert_eq!(spec.routes[0].target_port, 6379);
        } else {
            assert!(true);
        }
        assert_eq!(config.tls.enabled, false);
        Ok(())
    }
}
