fn extract_path(body: &[u8]) -> &str {
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

fn replace_bytes(data: Vec<u8>, search: Vec<u8>, replacement: Vec<u8>) -> Vec<u8> {
    data.windows(search.len())
        .enumerate()
        .find(|(_, window)| *window == search)
        .map(|(i, _)| {
            let mut new_data = data.clone();
            new_data.splice(i..i + search.len(), replacement.iter().copied());
            new_data
        })
        .unwrap_or(data)
}
