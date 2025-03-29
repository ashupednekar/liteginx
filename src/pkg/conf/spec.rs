use serde::Deserialize;


#[derive(Debug, Clone, Deserialize)]
pub struct Target{
    pub host: String,
    pub port: i32
}

#[derive(Debug, Clone, Deserialize)]
pub enum RouteKind{
    Tcp,
    Http{
        path: String,
        rewrite: Option<String>
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Route{
    name: String,
    listen: i32,
    kind: RouteKind,
    targets: Vec<Target>
}

#[derive(Debug, Clone, Deserialize)]
pub struct Tls{
    pub enabled: bool
}

#[derive(Debug, Clone, Deserialize)]
pub struct Spec{
    pub name: String,
    pub tls: Tls 
}


