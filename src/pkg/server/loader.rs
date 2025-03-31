use crate::prelude::Result;
use matchit::Router;
use std::{
    collections::HashMap,
    env,
    fmt::{self, Display},
    fs,
};

use crate::pkg::conf::spec::Spec;

use super::Server;

impl Server {
    pub async fn new() -> Result<Server> {
        let config_path =
            env::var("LITEGINX_CONF_DIR").unwrap_or(format!("{}/.config/liteginx", env!("HOME")));
        let configs: Vec<Spec> = fs::read_dir(&config_path)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "yaml"))
            .filter_map(|yaml_path| fs::read_to_string(yaml_path.path()).ok())
            .filter_map(|yaml| serde_yaml::from_str::<Spec>(&yaml).ok())
            .collect();

        let mut server = Server {
            routes: 
        };
        for config in configs {
        }
        tracing::debug!("loaded config: {:#?}", &server);
        Ok(server)
    }
}

impl Display for Server {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "\nState Configuration:")?;
        writeln!(f, "===================")?;

        writeln!(f, "Routes:\n")?;
        for (port, routes) in &self.routes {
            writeln!(f, "Listen at Port: {}", port)?;
            for route in routes {
                writeln!(
                    f,
                    "   route to -> {}:{}",
                    route.target_host, route.target_port
                )?;
            }
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
