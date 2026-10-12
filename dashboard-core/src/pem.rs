//! PEM -> DER decoding for pinned CA certificates (`no_std`, zero-alloc).
//!
//! `config::ODOO_CA_CERT` is stored as a PEM string (like the C firmware's
//! `ODOO_CA_CERT`), but `embedded-tls`'s `rustpki` certificate verifier
//! consumes raw DER bytes. This decodes the base64 body of the first PEM
//! block in a buffer the caller owns.

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

fn decode_base64_char(c: u8) -> Option<u8> {
    match c {
        b'A'..=b'Z' => Some(c - b'A'),
        b'a'..=b'z' => Some(c - b'a' + 26),
        b'0'..=b'9' => Some(c - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

/// Decode the first PEM block in `pem` (a `-----BEGIN ...-----` /
/// `-----END ...-----` pair wrapping base64) into DER bytes written to
/// `out`. Returns the number of DER bytes written, or `None` if `pem` is
/// not a well-formed PEM block, contains invalid base64, or `out` is too
/// small to hold the decoded certificate.
pub fn pem_to_der(pem: &str, out: &mut [u8]) -> Option<usize> {
    let bytes = pem.as_bytes();
    let begin = find(bytes, b"-----BEGIN ")?;
    let header_nl = begin + bytes[begin..].iter().position(|&b| b == b'\n')?;
    let end = header_nl + 1 + find(&bytes[header_nl + 1..], b"-----END ")?;
    let body = &bytes[header_nl + 1..end];

    let mut out_len = 0usize;
    let mut acc: u32 = 0;
    let mut bits = 0u32;
    for &b in body {
        if b == b'\r' || b == b'\n' || b == b' ' || b == b'\t' {
            continue;
        }
        if b == b'=' {
            break;
        }
        let v = decode_base64_char(b)?;
        acc = (acc << 6) | u32::from(v);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            if out_len >= out.len() {
                return None;
            }
            // Intentional truncation to the low byte of the shift
            // accumulator - this is exactly how base64 byte extraction
            // works (bits [bits, bits+8) of acc, discarding the rest).
            #[allow(clippy::cast_possible_truncation)]
            {
                out[out_len] = (acc >> bits) as u8;
            }
            out_len += 1;
        }
    }
    Some(out_len)
}
