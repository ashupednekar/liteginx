use crate::prelude::Result;
use spec::{config::IngressConf, routes::Route};

pub mod conf;
pub mod spec;
pub mod server;

async fn listen() -> Result<()> {
    let configs = IngressConf::new()?;
    let routes = Route::new(configs)?;
    // TODO: use joinhandles to spawn downstream servers
    // pass route tx to upstream listeners while spawning those
    Ok(())
}
