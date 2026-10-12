//! SNTP client (port of `src/net/net_time.c`'s lwIP SNTP usage).
//!
//! lwIP's SNTP module runs as a background service and calls back into
//! `net_time_sntp_set` whenever a reply lands; this port instead does one
//! explicit request/response round trip per call, which `main`'s
//! `time_boot` polls on a timer until it succeeds (mirrors the C firmware's
//! `net_time_synced()` wait loop).
use embassy_net::dns::DnsQueryType;
use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_net::Stack;
use embassy_time::{with_timeout, Duration};

use crate::config::NTP_SERVER;

/// NTP epoch (1900-01-01) to Unix epoch (1970-01-01) offset, in seconds.
const NTP_UNIX_EPOCH_DELTA: u32 = 2_208_988_800;
const NTP_PORT: u16 = 123;
/// Overall per-attempt deadline (DNS + request + reply), matching the
/// brief's 10 s SNTP timeout.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// Minimum accepted SNTP reply length: through the transmit timestamp
/// field (offset 40..44).
const MIN_REPLY_LEN: usize = 44;

/// Query `config::NTP_SERVER` once over UDP/123 and return the server's
/// Unix time in seconds, or `None` on any DNS/network error, malformed
/// reply, or the 10 s timeout.
pub async fn query(stack: Stack<'static>) -> Option<u32> {
    with_timeout(REQUEST_TIMEOUT, query_inner(stack))
        .await
        .ok()
        .flatten()
}

async fn query_inner(stack: Stack<'static>) -> Option<u32> {
    let addrs = stack.dns_query(NTP_SERVER, DnsQueryType::A).await.ok()?;
    let server = *addrs.first()?;

    let mut rx_meta = [PacketMetadata::EMPTY; 4];
    let mut rx_buffer = [0u8; 128];
    let mut tx_meta = [PacketMetadata::EMPTY; 4];
    let mut tx_buffer = [0u8; 128];
    let mut socket = UdpSocket::new(
        stack,
        &mut rx_meta,
        &mut rx_buffer,
        &mut tx_meta,
        &mut tx_buffer,
    );
    socket.bind(0).ok()?;

    // 48-byte SNTP client request: first byte (LI=0, VN=4, Mode=3 client)
    // is the only field servers require to be set.
    let mut request = [0u8; 48];
    request[0] = 0x1B;
    socket.send_to(&request, (server, NTP_PORT)).await.ok()?;

    let mut reply = [0u8; 128];
    loop {
        let (n, meta) = socket.recv_from(&mut reply).await.ok()?;
        if meta.endpoint.addr != server {
            continue;
        }
        if n < MIN_REPLY_LEN {
            continue;
        }
        let mode = reply[0] & 0x07;
        if mode != 4 && mode != 5 {
            // Not a server (4) or broadcast (5) reply; ignore and keep
            // waiting within the outer timeout.
            continue;
        }
        let secs = u32::from_be_bytes([reply[40], reply[41], reply[42], reply[43]]);
        return secs.checked_sub(NTP_UNIX_EPOCH_DELTA);
    }
}
