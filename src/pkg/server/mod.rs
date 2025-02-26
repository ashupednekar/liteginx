use async_trait::async_trait;
use matchit::Router;
use std::collections::HashMap;

use crate::{
    pkg::conf::spec::{HttpRoute, TcpRoute},
    prelude::Result,
};

mod http;
mod loader;
mod tcp;

pub type TcpRoutes = HashMap<i32, Vec<TcpRoute>>;
pub type HttpRoutes = HashMap<i32, Router<Vec<HttpRoute>>>;

#[derive(Debug)]
struct Server {
    tcp_routes: TcpRoutes,
    http_routes: HttpRoutes,
}

impl Server {
    async fn start(&self) {}
}


#[async_trait]
pub trait ForwardRoutes{
    async fn forward(&self, body: Vec<u8>) -> Result<()>;
}
