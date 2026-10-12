use dashboard_core::pem;

const PEM: &str = "-----BEGIN CERTIFICATE-----\nSGVsbG8sIHdvcmxkIQ==\n-----END CERTIFICATE-----\n";

#[test]
fn decodes_known_vector() {
    let mut out = [0u8; 32];
    let n = pem::pem_to_der(PEM, &mut out).unwrap();
    assert_eq!(&out[..n], b"Hello, world!");
}

#[test]
fn ignores_embedded_line_wrapping() {
    let wrapped =
        "-----BEGIN CERTIFICATE-----\nSGVsbG8s\r\nIHdvcmxkIQ==\n-----END CERTIFICATE-----\n";
    let mut out = [0u8; 32];
    let n = pem::pem_to_der(wrapped, &mut out).unwrap();
    assert_eq!(&out[..n], b"Hello, world!");
}

#[test]
fn rejects_missing_markers() {
    let mut out = [0u8; 32];
    assert!(pem::pem_to_der("not a pem", &mut out).is_none());
}

#[test]
fn rejects_buffer_too_small() {
    let mut out = [0u8; 4];
    assert!(pem::pem_to_der(PEM, &mut out).is_none());
}

#[test]
fn rejects_invalid_base64() {
    let bad = "-----BEGIN CERTIFICATE-----\n!!!not-base64!!!\n-----END CERTIFICATE-----\n";
    let mut out = [0u8; 32];
    assert!(pem::pem_to_der(bad, &mut out).is_none());
}
