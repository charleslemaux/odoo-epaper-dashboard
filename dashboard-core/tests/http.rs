use dashboard_core::http;

#[test]
fn build_request() {
    let mut buf = [0u8; 512];
    let len = http::build_request(&mut buf, "odoo.test.lan", "/jsonrpc", "{}").unwrap();
    let req = std::str::from_utf8(&buf[..len]).unwrap();
    assert!(req.starts_with("POST /jsonrpc HTTP/1.0\r\n"));
    assert!(req.contains("Host: odoo.test.lan\r\n"));
    assert!(req.contains("Content-Type: application/json\r\n"));
    assert!(req.contains("Content-Length: 2\r\n"));
    assert!(req.contains("Connection: close\r\n\r\n{}"));
}

#[test]
fn build_truncation() {
    let mut buf = [0u8; 16];
    assert!(http::build_request(&mut buf, "odoo.test.lan", "/jsonrpc", "{}").is_none());
}

#[test]
fn parse_status() {
    assert_eq!(
        http::parse_status(b"HTTP/1.0 200 OK\r\n\r\nbody"),
        Some(200)
    );
    assert_eq!(
        http::parse_status(b"HTTP/1.1 404 Not Found\r\n\r\n"),
        Some(404)
    );
    assert_eq!(http::parse_status(b"garbage"), None);
}

#[test]
fn body_offset() {
    let resp = b"HTTP/1.0 200 OK\r\nA: b\r\n\r\n{\"x\":1}";
    assert_eq!(http::body_offset(resp), Some(25));
    assert_eq!(&resp[25..], b"{\"x\":1}");
    assert_eq!(http::body_offset(b"no separator"), None);
}
