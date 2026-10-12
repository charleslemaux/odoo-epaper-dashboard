//! Panel init sequence, refresh cycle and deep sleep.
//!
//! Direct transcription of `src/epd/epd_io.c` + `src/epd/epd_driver.c`
//! (paced bit-bang SPI transport, mode 0, MSB first) into one module.

use dashboard_core::gfx::Frame;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::{PIN_10, PIN_11, PIN_12, PIN_13, PIN_8, PIN_9};
use embassy_rp::watchdog::Watchdog;
use embassy_rp::Peri;
use embassy_time::{block_for, Duration, Instant};

use crate::config::EPD_CLK_HALF_PERIOD_US;

/// Reload value fed to the watchdog from this module.
///
/// `epd_io.c`/`epd_driver.c` only call the parameterless pico-sdk
/// `watchdog_update()`, which reloads the counter to whatever period was
/// passed to `watchdog_enable()` in `src/app/main.c` (`watchdog_enable(8000,
/// 1)`). `embassy_rp::watchdog::Watchdog::feed` takes an explicit timeout
/// instead, so this constant must track that 8000 ms figure.
pub(crate) const WDT_RELOAD: Duration = Duration::from_millis(8000);

/// `epd_io_wait_idle` timeout used after reset, before the init sequence.
const SHORT_TIMEOUT_MS: u32 = 5_000;
/// `epd_io_wait_idle` timeout used after every command in the refresh path.
const LONG_TIMEOUT_MS: u32 = 60_000;
/// `power_on_reacts` window waiting for BUSY to assert after command 0x04.
const REACTION_WINDOW_MS: u64 = 3_000;

/// The exact 13-entry init sequence from `epd_driver.c`'s `INIT_SEQUENCE`.
const INIT_SEQUENCE: &[(u8, &[u8])] = &[
    (0xAA, &[0x49, 0x55, 0x20, 0x08, 0x09, 0x18]),
    (0x01, &[0x3F]),
    (0x00, &[0x5F, 0x69]),
    (0x03, &[0x00, 0x54, 0x00, 0x44]),
    (0x05, &[0x40, 0x1F, 0x1F, 0x2C]),
    (0x06, &[0x6F, 0x1F, 0x17, 0x49]),
    (0x08, &[0x6F, 0x1F, 0x1F, 0x22]),
    (0x30, &[0x03]),
    (0x50, &[0x3F]),
    (0x60, &[0x02, 0x00]),
    (0x61, &[0x03, 0x20, 0x01, 0xE0]),
    (0x84, &[0x01]),
    (0xE3, &[0x2F]),
];

/// Waveshare 7.3" e-Paper HAT (E) Spectra 6 panel over paced bit-bang SPI.
pub struct Epd<'d> {
    rst: Output<'d>,
    dc: Output<'d>,
    cs: Output<'d>,
    clk: Output<'d>,
    din: Output<'d>,
    busy: Input<'d>,
}

impl<'d> Epd<'d> {
    /// Build the driver and initialize the GPIO lines (port of `epd_io_init`).
    pub fn new(
        rst: Peri<'d, PIN_12>,
        dc: Peri<'d, PIN_8>,
        cs: Peri<'d, PIN_9>,
        clk: Peri<'d, PIN_10>,
        din: Peri<'d, PIN_11>,
        busy: Peri<'d, PIN_13>,
    ) -> Self {
        let cs = Output::new(cs, Level::High);
        let clk = Output::new(clk, Level::Low);
        let dc = Output::new(dc, Level::Low);
        let din = Output::new(din, Level::Low);
        let rst = Output::new(rst, Level::Low);
        let busy = Input::new(busy, Pull::Up);
        Self {
            rst,
            dc,
            cs,
            clk,
            din,
            busy,
        }
    }

    /// Port of `epd_io_reset`: 1(20ms) / 0(2ms) / 1(20ms).
    fn reset(&mut self) {
        self.rst.set_high();
        block_for(Duration::from_millis(20));
        self.rst.set_low();
        block_for(Duration::from_millis(2));
        self.rst.set_high();
        block_for(Duration::from_millis(20));
    }

    /// Port of `tx_byte`: MSB-first, clock-half-period paced before and
    /// between edges.
    fn tx_byte(&mut self, byte: u8) {
        let half = Duration::from_micros(EPD_CLK_HALF_PERIOD_US as u64);
        for bit in (0..8).rev() {
            if (byte >> bit) & 1 == 1 {
                self.din.set_high();
            } else {
                self.din.set_low();
            }
            block_for(half);
            self.clk.set_high();
            block_for(half);
            self.clk.set_low();
        }
    }

    /// Port of `epd_io_command`.
    fn command(&mut self, cmd: u8) {
        self.dc.set_low();
        self.cs.set_low();
        self.tx_byte(cmd);
        self.cs.set_high();
    }

    /// Port of `epd_io_data`: watchdog fed every 256 bytes.
    fn data(&mut self, data: &[u8], wdt: &mut Watchdog) {
        self.dc.set_high();
        self.cs.set_low();
        for (i, &byte) in data.iter().enumerate() {
            self.tx_byte(byte);
            if i & 0xFF == 0 {
                wdt.feed(WDT_RELOAD);
            }
        }
        self.cs.set_high();
    }

    /// Port of `epd_io_wait_idle`: BUSY low = busy.
    fn wait_idle(&mut self, timeout_ms: u32, wdt: &mut Watchdog) -> Result<(), ()> {
        let deadline = Instant::now() + Duration::from_millis(timeout_ms as u64);
        block_for(Duration::from_millis(30));
        while self.busy.is_low() {
            if Instant::now() >= deadline {
                return Err(());
            }
            wdt.feed(WDT_RELOAD);
            block_for(Duration::from_millis(10));
        }
        Ok(())
    }

    /// Port of `send_init_sequence`.
    fn send_init_sequence(&mut self, wdt: &mut Watchdog) {
        for &(cmd, data) in INIT_SEQUENCE {
            self.command(cmd);
            self.data(data, wdt);
        }
    }

    /// Port of `power_on_reacts`.
    fn power_on_reacts(&mut self, wdt: &mut Watchdog) -> Result<(), ()> {
        let deadline = Instant::now() + Duration::from_millis(REACTION_WINDOW_MS);
        self.command(0x04);
        while Instant::now() < deadline {
            if self.busy.is_low() {
                return self.wait_idle(LONG_TIMEOUT_MS, wdt);
            }
            wdt.feed(WDT_RELOAD);
            block_for(Duration::from_millis(2));
        }
        Err(())
    }

    /// Port of `epd_init`: 3 reset+init attempts, power-on-reaction check.
    pub fn init(&mut self, wdt: &mut Watchdog) -> Result<(), ()> {
        for _ in 0..3 {
            self.reset();
            // C ignores this call's return value too; it only paces the
            // sequence, the real gate is `power_on_reacts` below.
            let _ = self.wait_idle(SHORT_TIMEOUT_MS, wdt);
            self.send_init_sequence(wdt);
            if self.power_on_reacts(wdt).is_ok() {
                return Ok(());
            }
            block_for(Duration::from_millis(200));
        }
        Err(())
    }

    /// Port of `turn_on_display`.
    fn turn_on_display(&mut self, wdt: &mut Watchdog) -> Result<(), ()> {
        self.command(0x04);
        self.wait_idle(LONG_TIMEOUT_MS, wdt)?;
        self.command(0x12);
        self.data(&[0x00], wdt);
        self.wait_idle(LONG_TIMEOUT_MS, wdt)?;
        self.command(0x02);
        self.data(&[0x00], wdt);
        self.wait_idle(LONG_TIMEOUT_MS, wdt)
    }

    /// Port of `epd_display`: 0x10 + buffer, then the turn-on sequence.
    pub fn display(&mut self, fb: &Frame, wdt: &mut Watchdog) -> Result<(), ()> {
        self.command(0x10);
        self.data(fb, wdt);
        self.turn_on_display(wdt)
    }

    /// Port of `epd_sleep`: 0x07 0xA5.
    ///
    /// `epd_io_data` would feed the watchdog once for this single byte (its
    /// `i == 0` check); this frozen signature takes no `Watchdog`, and the
    /// single-byte transfer is microseconds long against an 8 s reload
    /// period, so the feed is dropped here rather than threading a watchdog
    /// reference through for one immaterial checkpoint.
    pub fn sleep(&mut self) {
        self.command(0x07);
        self.dc.set_high();
        self.cs.set_low();
        self.tx_byte(0xA5);
        self.cs.set_high();
        block_for(Duration::from_millis(20));
    }
}
