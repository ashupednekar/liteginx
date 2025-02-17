use crate::prelude::Result;
use matchit::Router;
use std::{collections::HashMap, env, fs};

use super::spec::{Config, HttpRoute, Spec, TcpRoute};

#[derive(Debug)]
struct State {
    tcp_routes: HashMap<i32, Vec<TcpRoute>>,
    http_routes: Router<Vec<HttpRoute>>,
}

impl State {
    fn new() -> State {
        State {
            tcp_routes: HashMap::new(),
            http_routes: Router::new(),
        }
    }

    fn load() -> Result<State> {
        let config_path =
            env::var("LITEGINX_CONF_DIR").unwrap_or(format!("{}/.config/liteginx", env!("HOME")));
        Ok(fs::read_dir(&config_path)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "yaml"))
            .filter_map(|yaml_path| fs::read_to_string(yaml_path.path()).ok())
            .filter_map(|yaml| serde_yaml::from_str::<Config>(&yaml).ok())
            .fold(Self::new(), |mut state, config| {
                match config.spec {
                    Spec::Tcp(spec) => {
                        state.tcp_routes.insert(
                            spec.listen_port,
                            spec.routes
                        );
                    }
                    Spec::Http(spec) => {
                        state.http_routes.insert(spec.path, spec.routes).ok();
                    }
                }
                state
            }))
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use super::*;

    #[test]
    #[traced_test]
    fn test_load_state() {
        let state = State::load();
        tracing::debug!("state: {:?}", &state);

    }
}
