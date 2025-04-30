use std::time::Duration;

use config::{Config, ConfigError, Environment};
use humantime::parse_duration;
use lazy_static::lazy_static;
use serde::{Deserialize, Deserializer};

#[derive(Deserialize)]
pub struct Settings {
    pub liteginx_conf_dir: String,
    pub upstream_reconnect_heartbeat: String 
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let conf = Config::builder()
            .add_source(Environment::default())
            .build()?;
        conf.try_deserialize()
    }
}

lazy_static! {
    pub static ref settings: Settings = Settings::new().expect("improperly configured");
}
