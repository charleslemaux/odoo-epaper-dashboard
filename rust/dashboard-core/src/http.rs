//! HTTP/1.0 request building and response splitting (port of src/net/http_util.c).

use crate::bufwrite::SliceWriter;
use core::fmt::Write;

pub fn build_request(dst: &mut [u8], host: &str, path: &str, body: &str) -> Option<usize> {
    let mut w = SliceWriter::new(dst);
    write!(
        w,
        "POST {path} HTTP/1.0\r\nHost: {host}\r\nContent-Type: application/json\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
    .ok()?;
    Some(w.len())
}

pub fn parse_status(resp: &[u8]) -> Option<u16> {
    if resp.len() < 12 || !resp.starts_with(b"HTTP/1.") {
        return None;
    }
    let mut status = 0u16;
    for &b in &resp[9..12] {
        if !b.is_ascii_digit() {
            return None;
        }
        status = status * 10 + (b - b'0') as u16;
    }
    Some(status)
}

pub fn body_offset(resp: &[u8]) -> Option<usize> {
    resp.windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|i| i + 4)
}
