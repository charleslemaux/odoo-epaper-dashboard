//! One-shot HTTPS POST client over embedded-tls (TLS 1.3) + embassy-net.
//!
//! Port of `src/net/http_client.c` + `src/net/http_conn.c`'s lwIP
//! `altcp_tls` client: resolve DNS, open a single fresh TCP+TLS connection
//! (`Connection: close`, one connection per call), send the request built
//! by `dashboard_core::http::build_request`, read the response until the
//! peer closes (mirroring the C firmware's `cb_recv(p == NULL)` completion
//! signal) or the response buffer fills, then parse status + body.
//!
//! Certificate verification: when `config::ODOO_CA_CERT` is `None` (the
//! default, like the C firmware's `altcp_tls_create_config_client(0, 0)`),
//! the connection is unverified. When it is `Some(pem)`, the PEM is decoded
//! to DER (`dashboard_core::pem`) and pinned via embedded-tls's `rustpki`
//! `CertVerifier` (chain-of-trust + hostname checks; `NoClock` skips the
//! certificate expiry check since no wall clock is threaded through this
//! one-shot call). If the PEM fails to decode, the connection proceeds
//! unverified and logs a prominent warning rather than failing the request.
//!
use dashboard_core::http::{body_offset, build_request, parse_status};
use dashboard_core::pem;
use embassy_net::dns::DnsQueryType;
use embassy_net::tcp::TcpSocket;
use embassy_net::{IpAddress, Stack};
use embassy_rp::peripherals::TRNG;
use embassy_rp::trng::{self, Trng};
use embassy_rp::{bind_interrupts, Peri};
use embassy_time::{with_timeout, Duration};
use embedded_tls::pki::CertVerifier;
use embedded_tls::{
    Aes128GcmSha256, Certificate, CryptoProvider, CryptoRngCore, NoClock, TlsConfig, TlsConnection,
    TlsContext, TlsError, TlsVerifier, UnsecureProvider,
};

use crate::config::{ODOO_CA_CERT, ODOO_HOST, ODOO_PORT};

bind_interrupts!(struct Irqs {
    TRNG_IRQ => trng::InterruptHandler<TRNG>;
});

/// Build the hardware RNG used for the TLS handshake. Call once at boot and
/// keep the `Trng` alive across repeated `post_json` calls.
pub fn new_rng(peripheral: Peri<'static, TRNG>) -> Trng<'static, TRNG> {
    Trng::new(peripheral, Irqs, trng::Config::default())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetErr {
    Dns,
    Connect,
    Tls,
    Io,
    Http,
}

#[derive(Debug)]
pub struct Response<'a> {
    // Always 200 by the time a `Response` is returned (non-200 is mapped to
    // `NetErr::Http` in `post_json`); kept for parity with the C firmware's
    // `http_response.status` and for callers that may want to log it.
    #[allow(dead_code)]
    pub status: u16,
    pub body: &'a [u8],
}

const DNS_TIMEOUT: Duration = Duration::from_secs(10);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const EXCHANGE_TIMEOUT: Duration = Duration::from_secs(30);

const RESP_BUF_LEN: usize = 16384;
const REQUEST_BUF_LEN: usize = 4096;
const TCP_BUF_LEN: usize = 4096;
const TLS_RECORD_LEN: usize = 16640;
/// Max DER-encoded CA certificate size accepted for pinning.
const CA_DER_LEN: usize = 4096;

/// Declares a function returning a `&'static mut` reference to a
/// zero-initialized buffer, kept out of the (small) task stack/future.
///
/// SAFETY: every buffer this generates is only ever borrowed for the
/// duration of one `post_json` call, which runs a single fresh connection
/// to completion before the firmware's poll loop calls it again, so the
/// `&'static mut` handed out here is never aliased.
macro_rules! static_buf {
    ($name:ident, $len:expr) => {
        fn $name() -> &'static mut [u8; $len] {
            static mut BUF: [u8; $len] = [0; $len];
            unsafe { &mut *core::ptr::addr_of_mut!(BUF) }
        }
    };
}

static_buf!(request_buf, REQUEST_BUF_LEN);
static_buf!(tcp_rx_buf, TCP_BUF_LEN);
static_buf!(tcp_tx_buf, TCP_BUF_LEN);
static_buf!(tls_read_record_buf, TLS_RECORD_LEN);
static_buf!(tls_write_record_buf, TLS_RECORD_LEN);
static_buf!(ca_der_buf, CA_DER_LEN);

/// Map an embedded-tls error to `NetErr`: `TlsError::Io` wraps a genuine
/// transport failure reported by the TCP socket, everything else is a TLS
/// protocol-level failure.
fn map_tls_err(e: TlsError) -> NetErr {
    match e {
        TlsError::Io(_) => NetErr::Io,
        _ => NetErr::Tls,
    }
}

async fn resolve(stack: Stack<'static>) -> Option<IpAddress> {
    let addrs = stack.dns_query(ODOO_HOST, DnsQueryType::A).await.ok()?;
    addrs.first().copied()
}

struct PinnedProvider<'r> {
    rng: &'r mut Trng<'static, TRNG>,
    verifier: CertVerifier<'r, Aes128GcmSha256, NoClock, CA_DER_LEN>,
}

impl CryptoProvider for PinnedProvider<'_> {
    type CipherSuite = Aes128GcmSha256;
    type Signature = &'static [u8];

    fn rng(&mut self) -> impl CryptoRngCore {
        &mut *self.rng
    }

    fn verifier(&mut self) -> Result<&mut impl TlsVerifier<Self::CipherSuite>, TlsError> {
        Ok(&mut self.verifier)
    }
}

/// Open the TLS session: pinned-CA verification when `ODOO_CA_CERT` decodes
/// cleanly, otherwise unverified (see module docs for the fallback policy).
async fn open(
    tls: &mut TlsConnection<'_, TcpSocket<'_>, Aes128GcmSha256>,
    tls_config: &TlsConfig<'_>,
    rng: &mut Trng<'static, TRNG>,
) -> Result<(), NetErr> {
    let Some(pem) = ODOO_CA_CERT else {
        let provider = UnsecureProvider::new::<Aes128GcmSha256>(rng);
        return tls
            .open(TlsContext::new(tls_config, provider))
            .await
            .map_err(map_tls_err);
    };

    let der_buf = ca_der_buf();
    let Some(der_len) = pem::pem_to_der(pem, der_buf) else {
        log::warn!(
            "tls: ODOO_CA_CERT is not valid PEM; proceeding WITHOUT certificate verification"
        );
        let provider = UnsecureProvider::new::<Aes128GcmSha256>(rng);
        return tls
            .open(TlsContext::new(tls_config, provider))
            .await
            .map_err(map_tls_err);
    };

    let verifier = CertVerifier::<Aes128GcmSha256, NoClock, CA_DER_LEN>::new(Certificate::X509(
        &der_buf[..der_len],
    ));
    let provider = PinnedProvider { rng, verifier };
    tls.open(TlsContext::new(tls_config, provider))
        .await
        .map_err(map_tls_err)
}

/// Open the connection, send `request`, then read into `resp_buf` until the
/// peer closes the session (clean TLS close-notify or a bare TCP EOF both
/// count as "response complete", matching the C firmware's
/// `cb_recv(p == NULL)`). Returns the number of bytes received.
///
/// A response that doesn't fit in `resp_buf` is a hard failure
/// (`NetErr::Io`), matching the C firmware's `cb_recv` (`copied <
/// p->tot_len` -> `phase = -3` -> the whole request fails in
/// `http_client.c`): filling the buffer without having observed a close is
/// never treated as "response complete", so a truncated body is never
/// returned as a successful response.
async fn exchange(
    tls: &mut TlsConnection<'_, TcpSocket<'_>, Aes128GcmSha256>,
    tls_config: &TlsConfig<'_>,
    rng: &mut Trng<'static, TRNG>,
    request: &[u8],
    resp_buf: &mut [u8],
) -> Result<usize, NetErr> {
    open(tls, tls_config, rng).await?;

    let mut sent = 0usize;
    while sent < request.len() {
        sent += tls.write(&request[sent..]).await.map_err(map_tls_err)?;
    }
    tls.flush().await.map_err(map_tls_err)?;

    let mut received = 0usize;
    let mut closed = false;
    while received < resp_buf.len() {
        match tls.read(&mut resp_buf[received..]).await {
            Ok(0) => {
                closed = true;
                break;
            }
            Ok(n) => received += n,
            Err(TlsError::IoError | TlsError::ConnectionClosed) => {
                closed = true;
                break;
            }
            Err(e) => return Err(map_tls_err(e)),
        }
    }
    if !closed {
        // Either the buffer filled without the peer closing, or it filled
        // on the very read that would have closed next: both look
        // identical from here (no probe read), and both are treated as
        // overflow, same as the C firmware.
        return Err(NetErr::Io);
    }
    Ok(received)
}

/// Resolve `config::ODOO_HOST`, open one fresh TCP+TLS 1.3 connection
/// (SNI = `ODOO_HOST`), POST `body` to `path`, and return the parsed
/// response. `resp_buf` backs the returned `Response::body` slice.
pub async fn post_json<'a>(
    stack: Stack<'static>,
    rng: &mut Trng<'static, TRNG>,
    path: &str,
    body: &str,
    resp_buf: &'a mut [u8; RESP_BUF_LEN],
) -> Result<Response<'a>, NetErr> {
    let req = request_buf();
    let req_len = build_request(req, ODOO_HOST, path, body).ok_or(NetErr::Http)?;

    let addr = with_timeout(DNS_TIMEOUT, resolve(stack))
        .await
        .map_err(|_| NetErr::Dns)?
        .ok_or(NetErr::Dns)?;

    let mut socket = TcpSocket::new(stack, tcp_rx_buf(), tcp_tx_buf());
    with_timeout(CONNECT_TIMEOUT, socket.connect((addr, ODOO_PORT)))
        .await
        .map_err(|_| NetErr::Connect)?
        .map_err(|_| NetErr::Connect)?;

    let tls_config = TlsConfig::new().with_server_name(ODOO_HOST);
    let mut tls = TlsConnection::new(socket, tls_read_record_buf(), tls_write_record_buf());

    let received = with_timeout(
        EXCHANGE_TIMEOUT,
        exchange(&mut tls, &tls_config, rng, &req[..req_len], resp_buf),
    )
    .await
    .map_err(|_| NetErr::Tls)??;

    let status = parse_status(&resp_buf[..received]).ok_or(NetErr::Http)?;
    if status != 200 {
        return Err(NetErr::Http);
    }
    let offset = body_offset(&resp_buf[..received]).ok_or(NetErr::Http)?;
    Ok(Response {
        status,
        body: &resp_buf[offset..received],
    })
}
