use std::sync::Arc;

use crate::prelude::{ProxyError, Result};
use server::downstream::ListenDownstream;
use spec::{config::IngressConf, routes::Route};
use tokio::task::JoinSet;

pub mod conf;
pub mod server;
pub mod spec;

pub async fn listen() -> Result<()> {
    let configs = IngressConf::new()?;
    let routes = Route::new(configs)?;
    let set = routes.iter().fold(JoinSet::new(), |mut set, route| {
        let route = Arc::clone(&route);
        set.spawn(async move {
            route.serve().await?;
            Ok::<(), ProxyError>(())
        });
        set
    });
    tokio::select! {
        _ = set.join_all() => {},
        _ = tokio::signal::ctrl_c() => {}
    };
    Ok(())
}
