use matchit::Router;
use serde_json::json;

use crate::pkg::{conf::settings, Result, spec::routes::Endpoint};



pub fn extract_path(body: &[u8]) -> &str {
    let mut lines = body.split(|&b| b == b'\r' || b == b'\n');
    if let Some(request_line) = lines.next() {
        let mut parts = request_line.splitn(3, |&b| b == b' ');
        parts.next();
        if let Some(uri) = parts.next() {
            let path = std::str::from_utf8(uri).unwrap_or("/");
            return path.strip_prefix('/').unwrap_or(path);
        }
    }
    ""
}

pub fn match_prefix<'a>(router: &'a Router<Endpoint>, path: &str) -> Option<&'a Endpoint> {
    let mut parts: Vec<&str> = path.trim_end_matches('/').split('/').collect();

    while !parts.is_empty() {
        let try_path = format!("/{}", parts.join("/"));
        if let Ok(m) = router.at(&try_path) {
            return Some(m.value);
        }
        parts.pop();
    }
    None
}

pub fn rewrite_path(data: &[u8], search: Vec<u8>, replacement: Vec<u8>) -> Vec<u8> {
    data.windows(search.len())
        .enumerate()
        .find(|(_, window)| *window == search)
        .map(|(i, _)| {
            let mut new_data = data.to_vec();
            new_data.splice(i..i + search.len(), replacement.iter().copied());
            new_data
        })
        .unwrap_or_else(|| data.to_vec())
}

pub fn http_404_response() -> Result<String> {
    let body = serde_json::to_string(&json!({
        "detail": &settings.not_found_message.clone().unwrap_or("not found".into())
    }))?;
    let content_length = body.len();
    Ok(format!(
        "HTTP/1.1 404 Not Found\r\n\
        Content-Type: application/json\r\n\
        Content-Length: {}\r\n\
        Connection: close\r\n\
        \r\n\
        {}",
        content_length, body
    ))
}
