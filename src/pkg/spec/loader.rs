use std::{collections::HashMap, fs, sync::Arc};

use matchit::Router;

use super::{
    config::{IngressConf, Kind},
    routes::{Endpoint, Route, UpstreamTarget},
};
use crate::{pkg::conf::settings, prelude::Result};

impl IngressConf {
    pub fn new() -> Result<Vec<IngressConf>> {
        Ok(fs::read_dir(&settings.liteginx_conf_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "yaml"))
            .filter_map(|yaml_path| fs::read_to_string(yaml_path.path()).ok())
            .filter_map(|yaml| serde_yaml::from_str::<IngressConf>(&yaml).ok())
            .collect())
    }
}

impl Route {
    pub fn new(configs: Vec<IngressConf>) -> Result<Vec<Arc<Route>>> {
        let paths: HashMap<u16, (Option<Router<Endpoint>>, Vec<UpstreamTarget>)> = configs
            .iter()
            .flat_map(|conf| {
                tracing::debug!("loading conf: {:?}", &conf.name);
                conf.spec.iter()
            })
            .fold(HashMap::new(), |mut paths, spec| {
                tracing::debug!("adding listener spec: {:?}", &spec);
                let entry = paths
                    .entry(spec.listen)
                    .or_insert_with(|| (None, spec.targets.clone()));
                if let Kind::Http = spec.kind {
                    let router = entry.0.get_or_insert_with(Router::new);
                    let path = spec
                        .path
                        .clone()
                        .expect("http spec missing mandatory field path".into());
                    if router.at(&path).is_ok() {
                        tracing::warn!("{} conflicts with existing endpoint", &path);
                        return paths;
                    }

                    let rewrite = spec.rewrite.clone();
                    if let Err(err) = router.insert(path.clone(), Endpoint { path, rewrite }) {
                        tracing::error!("Failed to insert: {}", err);
                        return paths;
                    }
                }
                spec.targets.iter().for_each(|target| {
                    if !entry.1.contains(target) {
                        entry.1.push(target.clone());
                    }
                });
                paths
            });
        let routes = paths
            .into_iter()
            .map(|(listen, (endpoints, targets))| Route {
                listen,
                endpoints,
                targets,
                ..Default::default()
            })
            .map(Arc::new)
            .collect();
        Ok(routes)
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use super::*;

    #[test]
    #[traced_test]
    fn test_load_http_test() -> Result<()> {
        let configs = IngressConf::new()?;
        let routes = Route::new(configs)?;

        let route = routes
            .iter()
            .find(|r| r.listen == 80 && r.endpoints.iter().any(|e| e.path == "/one"))
            .expect("Missing one-ingress route");

        assert_eq!(route.listen, 80);
        assert_eq!(route.endpoints.len(), 2);
        let ep = &route.endpoints[0];
        assert_eq!(ep.path, "/one");
        assert!(ep.rewrite.is_none());

        assert_eq!(route.targets.len(), 1);
        let target = &route.targets[0];
        assert_eq!(target.host, "localhost");
        assert_eq!(target.port, 3000);

        Ok(())
    }

    #[test]
    #[traced_test]
    fn load_http_with_rewrite_test() -> Result<()> {
        let configs = IngressConf::new()?;
        let routes = Route::new(configs)?;

        let route = routes
            .iter()
            .find(|r| r.listen == 80 && r.endpoints.iter().any(|e| e.path == "/two"))
            .expect("Missing two-ingress route");

        assert_eq!(route.listen, 80);
        assert_eq!(route.endpoints.len(), 2);
        let ep = &route.endpoints[1];
        assert_eq!(ep.path, "/two");
        assert_eq!(ep.rewrite.as_deref(), Some("/"));

        assert_eq!(route.targets.len(), 1);
        let target = &route.targets[0];
        assert_eq!(target.host, "localhost");
        assert_eq!(target.port, 3000);

        Ok(())
    }

    #[test]
    #[traced_test]
    fn load_tcp() -> Result<()> {
        let configs = IngressConf::new()?;
        let routes = Route::new(configs)?;

        tracing::debug!("routes: {:?}", &routes);
        let route = routes
            .iter()
            .find(|r| r.listen == 4000)
            .expect("Missing tcptest-ingress route");

        assert!(route.endpoints.is_empty()); // No path or rewrite for TCP

        assert_eq!(route.targets.len(), 1);
        let target = &route.targets[0];
        assert_eq!(target.host, "localhost");
        assert_eq!(target.port, 4001);

        Ok(())
    }
}
