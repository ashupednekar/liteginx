use matchit::Router;
use serde::{Deserialize, Deserializer};
use tokio::sync::broadcast;


#[derive(Debug, Clone, Deserialize)]
pub struct Target{
    pub host: String,
    pub port: i32
}

#[derive(Debug, Clone, Deserialize)]
pub enum RouteKind{
    Tcp,
    Http{
        path: String,
        rewrite: Option<String>
    }
}

#[derive(Debug, Clone)]
pub struct Route{
    name: String,
    listen: i32,
    kind: RouteKind,
    targets: Vec<Target>,
    proxy_tx: broadcast::Sender<Vec<u8>>,
    upstream_tx: broadcast::Sender<Vec<u8>>
}


impl<'de> Deserialize<'de> for Route {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RouteHelper {
            name: String,
            listen: i32,
            kind: RouteKind,
            targets: Vec<Target>,
            target_host: String,
            target_port: i32,
        }

        let helper = RouteHelper::deserialize(deserializer)?;
        let (proxy_tx, _) = broadcast::channel::<Vec<u8>>(1);
        let (upstream_tx, _) = broadcast::channel::<Vec<u8>>(1);
        Ok(Route {
            name: helper.name,
            listen: helper.listen,
            kind: helper.kind,
            targets: helper.targets,
            proxy_tx,
            upstream_tx,
        })
    }
}

#[derive(Debug, Clone)]
pub enum Routes {
    List(Vec<Route>),
    Matcher(Router<Route>),
}

#[derive(Debug, Clone, Deserialize)]
pub struct Tls{
    pub enabled: bool
}

#[derive(Debug, Clone, Deserialize)]
pub struct Spec{
    pub name: String,
    pub routes: Vec<Route>,
    pub tls: Tls 
}


