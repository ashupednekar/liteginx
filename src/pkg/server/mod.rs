use std::collections::HashMap;
use async_trait::async_trait;
use matchit::Router;

use crate::{prelude::Result, pkg::conf::spec::{HttpRoute, TcpRoute}};

mod loader;
mod tcp;

pub type TcpRoutes = HashMap<i32, Vec<TcpRoute>>;
pub type HttpRoutes = HashMap<i32, Router<Vec<HttpRoute>>>;


#[derive(Debug)]
struct Server {
    tcp_routes: TcpRoutes,
    http_routes: HttpRoutes,
}

impl Server{
  
    async fn start(&self){
         
    }

}
