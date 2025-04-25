use std::{collections::HashMap, fs};

use crate::{pkg::conf::settings, prelude::Result};
use super::{config::{IngressConf, Kind}, routes::{Endpoint, Route, UpstreamTarget} };

impl IngressConf{
    pub fn new() -> Result<Vec<IngressConf>>{
        Ok(fs::read_dir(&settings.liteginx_conf_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "yaml"))
            .filter_map(|yaml_path| fs::read_to_string(yaml_path.path()).ok())
            .filter_map(|yaml| serde_yaml::from_str::<IngressConf>(&yaml).ok())
            .collect())
    }
}

impl Route{
    pub fn new(configs: Vec<IngressConf>) -> Result<Vec<Route>>{
        let paths: HashMap<u16, (Vec<Endpoint>, Vec<UpstreamTarget>)> = configs.iter()
            .flat_map(|conf| {
                tracing::debug!("loading conf: {:?}", &conf.name);
                conf.spec.iter()
            })
            .fold(HashMap::new(), |mut paths, spec| {
                if let Kind::Http = spec.kind{
                    let entry = paths
                        .entry(spec.listen)
                        .or_insert_with(|| (vec![], spec.targets.clone()));
                    if entry.0.iter().any(|endpoint| endpoint.path == spec.path){
                        tracing::warn!("{} conflicts with existing endpoint", &spec.path);
                        return paths
                    }
                    let path = spec.path.clone();
                    let rewrite = spec.rewrite.clone();
                    entry.0.push(Endpoint { path, rewrite });
                    entry.1.extend(spec.targets.clone());
                };
                paths
            });
        let routes: Vec<Route> = paths
            .into_iter()
            .map(|(listen, (endpoints, targets))| Route{listen, endpoints, targets})
            .collect();
        Ok(routes)
    }
}


#[cfg(test)]
mod tests{
    use tracing_test::traced_test;

    use super::*;
    
    #[test]
    #[traced_test]
    fn loader_test() -> Result<()>{
        let configs = IngressConf::new()?;
        tracing::debug!("configs: {:?}", &configs);
        let routes = Route::new(configs)?;
        tracing::debug!("routes: {:?}", &routes);
        Ok(())
    }
}
