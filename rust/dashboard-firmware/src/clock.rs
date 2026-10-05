//! Wall-clock anchor (port of `src/net/net_time.c`'s always-on-timer clock).
//!
//! The C firmware anchors wall-clock time in the RP2350's always-on timer
//! (`aon_timer_set_time`), which keeps ticking through light sleep. This
//! port instead anchors the SNTP-reported Unix time to the monotonic
//! `embassy_time::Instant` clock at the moment of sync: [`Clock::now_unix`]
//! adds elapsed monotonic time back onto that anchor. `Timer::after` (the
//! only sleep this firmware uses) does not stop `Instant`, so the two
//! clocks track each other exactly as C's did.
//!
use dashboard_core::time_fmt::{civil_from_days, Date};
use embassy_time::Instant;

use crate::config::TZ_OFFSET_MIN;

const SECS_PER_DAY: i64 = 86_400;

/// A Unix timestamp anchored to the monotonic clock at the instant it was
/// observed (port of `net_time_sntp_set` + `net_time_local`).
#[derive(Clone, Copy)]
pub struct Clock {
    unix_at_boot: u32,
    anchor: Instant,
}

impl Clock {
    /// Anchor `unix_secs` (as returned by [`crate::sntp::query`]) to "now".
    pub fn new(unix_secs: u32) -> Self {
        Self {
            unix_at_boot: unix_secs,
            anchor: Instant::now(),
        }
    }

    /// Current Unix time, in seconds (port of `aon_timer_get_time`'s
    /// `tv_sec`).
    pub fn now_unix(&self) -> u32 {
        let elapsed = Instant::now()
            .saturating_duration_since(self.anchor)
            .as_secs();
        (u64::from(self.unix_at_boot) + elapsed) as u32
    }

    /// Current local date and time of day, applying `config::TZ_OFFSET_MIN`
    /// (port of `net_time_local`: `ts.tv_sec + TZ_OFFSET_MIN * 60`, split
    /// via civil-date arithmetic instead of `gmtime_r`).
    pub fn now_local(&self) -> (Date, u8, u8) {
        let local = i64::from(self.now_unix()) + i64::from(TZ_OFFSET_MIN) * 60;
        let days = local.div_euclid(SECS_PER_DAY);
        let day_secs = local.rem_euclid(SECS_PER_DAY);
        let date = civil_from_days(days);
        let hour = (day_secs / 3600) as u8;
        let minute = ((day_secs % 3600) / 60) as u8;
        (date, hour, minute)
    }

    /// Monotonic uptime in seconds, independent of SNTP sync (port of C's
    /// `now_s()`, which drives the refresh-interval arithmetic).
    pub fn uptime_s(&self) -> u32 {
        Instant::now().as_secs() as u32
    }
}
