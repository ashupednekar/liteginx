use std::collections::HashMap;
use matchit::Router;

use crate::{conf::settings, pkg::conf::spec::{HttpRoute, TcpRoute}};

mod loader;


#[derive(Debug)]
struct Server {
    tcp_routes: HashMap<i32, Vec<TcpRoute>>,
    http_routes: HashMap<i32, Router<Vec<HttpRoute>>>,
    port: i32
}

impl Server{
  
    async fn start(&self){
        
    }

}
