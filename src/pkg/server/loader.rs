use crate::{pkg::conf::spec::ToTcp, prelude::Result};
use matchit::Router;
use std::{
    collections::HashMap,
    env,
    fmt::{self, Display},
    fs,
};

use crate::pkg::conf::spec::{Config, Spec};

use super::Server;

impl Server {
    pub async fn new() -> Result<Server> {
        let config_path =
            env::var("LITEGINX_CONF_DIR").unwrap_or(format!("{}/.config/liteginx", env!("HOME")));
        let configs: Vec<Config> = fs::read_dir(&config_path)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "yaml"))
            .filter_map(|yaml_path| fs::read_to_string(yaml_path.path()).ok())
            .filter_map(|yaml| serde_yaml::from_str::<Config>(&yaml).ok())
            .collect();

        let mut server = Server {
            tcp_routes: HashMap::new(),
            http_routes: HashMap::new(),
        };
        for config in configs {
            match config.spec {
                Spec::Http(spec) => {
                    server
                        .http_routes
                        .entry(spec.listen_port)
                        .or_insert_with(Router::new)
                        .insert(&format!("{}/{{*p}}", &spec.path[1..]), spec.routes.clone())?;
                    server
                        .tcp_routes
                        .entry(spec.listen_port)
                        .or_insert(spec.routes.iter().map(|r| r.to_tcp()).collect());
                }
                Spec::Tcp(spec) => {
                    server
                        .tcp_routes
                        .entry(spec.listen_port)
                        .or_insert(spec.routes);
                }
            }
        }
        tracing::debug!("loaded config: {:#?}", &server);
        Ok(server)
    }
}

impl Display for Server {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "\nState Configuration:")?;
        writeln!(f, "===================")?;

        writeln!(f, "TCP Routes:\n")?;
        for (port, routes) in &self.tcp_routes {
            writeln!(f, "Listen at Port: {}", port)?;
            for route in routes {
                writeln!(
                    f,
                    "   route to -> {}:{}",
                    route.target_host, route.target_port
                )?;
            }
        }
        writeln!(f, "\nHttp Routes:\n")?;
        for (port, router) in &self.http_routes {
            writeln!(f, "Listen at Port: {}\n   route as: {:?}", port, router)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use super::*;

    #[tokio::test]
    #[traced_test]
    async fn test_load_state() -> Result<()> {
        unsafe { std::env::set_var("LITEGINX_CONF_DIR", "src/pkg/conf/fixtures/liteginx") }
        let state = Server::new().await?;
        tracing::debug!("state: {}", &state);
        Ok(())
    }
}
