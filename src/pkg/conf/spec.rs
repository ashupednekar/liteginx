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
pub struct VirtualHost{
    pub host: String,
    pub port: i32
}

#[derive(Deserialize, Clone)]
pub struct Http{
    #[serde(default = "default_http_kind")]
    pub kind: String,
    pub path: String,
    pub listen: VirtualHost,
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
    #[serde(deserialize_with = "deserialize_spec")]
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


#[cfg(test)]
mod tests{
    use super::*;
    use crate::Result;

    #[test]
    fn test_normal_http_deserialize() -> Result<()>{
        let conf_yaml = "
name: one-ingress
spec:
  kind: http
  path: /one
  listen:
    host: localhost
    port: 80
  route:
    host: localhost
    port: 3000
tls: 
  enabled: false";
        let config: Config = serde_yaml::from_str(conf_yaml)?;
        assert_eq!(config.name, "one-ingress");
        if let Spec::Http(spec) = config.spec { 
            assert_eq!(spec.kind, "http");
            assert_eq!(spec.path, "/one");
            assert_eq!(spec.listen.host, "localhost");
            assert_eq!(spec.listen.port, 80);
            assert_eq!(spec.route.host, "localhost");
            assert_eq!(spec.route.port, 3000);
            assert_eq!(spec.route.rewrite, None);
        }else{
            assert!(true);
        }
        assert_eq!(config.tls.enabled, false);
        Ok(())
    }

    #[test]
    fn test_normal_http_deserialize_with_rewrite() -> Result<()>{
        let conf_yaml = "
name: one-ingress
spec:
  kind: http
  path: /one
  listen:
    host: localhost
    port: 80
  route:
    host: localhost
    port: 3000
    rewrite: /
tls: 
  enabled: false";
        let config: Config = serde_yaml::from_str(conf_yaml)?;
        assert_eq!(config.name, "one-ingress");
        if let Spec::Http(spec) = config.spec { 
            assert_eq!(spec.kind, "http");
            assert_eq!(spec.path, "/one");
            assert_eq!(spec.listen.host, "localhost");
            assert_eq!(spec.listen.port, 80);
            assert_eq!(spec.route.host, "localhost");
            assert_eq!(spec.route.port, 3000);
            assert_eq!(spec.route.rewrite, Some("/".to_string()));
        }else{
            assert!(true);
        }
        assert_eq!(config.tls.enabled, false);
        Ok(())
    }

    #[test]
    fn test_tcp_proxy() -> Result<()>{
        let conf_yaml = "
name: redis-ingress
spec:
  kind: http
  port: 6379
tls:
  enabled: false";
        let config: Config = serde_yaml::from_str(conf_yaml)?;
        assert_eq!(config.name, "one-ingress");
        if let Spec::Tcp(spec) = config.spec { 
  
        }else{
            assert!(true);
        }
        assert_eq!(config.tls.enabled, false);
        Ok(())
    }

}
