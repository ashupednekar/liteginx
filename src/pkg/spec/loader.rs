use std::{collections::HashMap, fs};

use crate::{pkg::conf::settings, prelude::Result};
use super::{config::{IngressConf, Kind}, routes::{Endpoint, Route, UpstreamTarget} };

impl IngressConf{
    pub fn new() -> Result<Vec<IngressConf>>{
        Ok(fs::read_dir(&settings.liteginx_conf_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "yaml"))
            .filter_map(|yaml_path| fs::read_to_string(yaml_path.path()).ok())
            .filter_map(|yaml| {
                serde_yaml::from_str::<IngressConf>(&yaml).ok()
            })
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
                tracing::debug!("adding listener spec: {:?}", &spec);
                let entry = paths
                    .entry(spec.listen)
                    .or_insert_with(|| (vec![], spec.targets.clone()));
                if let Kind::Http = spec.kind {
                    let path = spec.path.clone().expect("http spec missing mandatory field path".into());
                    if entry.0.iter().any(|endpoint| endpoint.path == path){
                        tracing::warn!("{} conflicts with existing endpoint", &path);
                        return paths
                    }
                    let rewrite = spec.rewrite.clone();
                    entry.0.push(Endpoint { path, rewrite });
                } 
                spec.targets.iter()
                    .for_each(|target| {
                        if !entry.1.contains(target){
                            entry.1.push(target.clone());
                        }
                    });
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

    use
        super::*;

    #[test]
    #[traced_test]
    fn test_load_http_test() -> Result<()> {
        let configs = IngressConf::new()?;
        let routes = Route::new(configs)?;
    
        let route = routes.iter()
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
    
        let route = routes.iter()
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
        let route = routes.iter()
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
