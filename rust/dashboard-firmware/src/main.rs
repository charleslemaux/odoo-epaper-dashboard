//! e-paper dashboard firmware (Embassy, RP2350).
#![no_std]
#![no_main]

mod clock;
mod config;
mod epd;
mod net;
mod net_http;
mod odoo_client;
mod panic_log;
mod sntp;

use core::pin::pin;
use core::ptr::addr_of_mut;

use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_net::Stack;
use embassy_rp::block::ImageDef;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_rp::watchdog::Watchdog;
use embassy_rp::{bind_interrupts, peripherals::USB};
use embassy_time::Timer;
use static_cell::StaticCell;

use dashboard_core::dashboard::{self, DashboardData};
use dashboard_core::gfx::{Frame, BUFFER_SIZE};
use dashboard_core::refresh::{refresh_needed, RefreshTimes, Snapshot};
use dashboard_core::time_fmt::{banner, hhmm};

use crate::clock::Clock;
use crate::config::POLL_INTERVAL_S;
use crate::epd::{Epd, WDT_RELOAD};

#[link_section = ".start_block"]
#[used]
pub static IMAGE_DEF: ImageDef = ImageDef::secure_exe();

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

/// Top-level application state owned by `main`'s superloop (port of
/// `struct app`). Kept in a `StaticCell` rather than a local: a value held
/// across this firmware's many `.await` points gets embedded into the
/// generated state machine of whatever awaits it, and `main`'s task arena
/// is not guaranteed to have room for two `Snapshot`s on top of everything
/// else; a `StaticCell` moves it out of that arena entirely.
#[derive(Default)]
struct App {
    displayed: Snapshot,
    current: Snapshot,
    uid: i32,
    fails: u32,
    has_displayed: bool,
    last_refresh_s: u32,
    offline_since: heapless::String<8>,
}

/// The 187.5 KB framebuffer. A plain `static` (not a local): its all-zero
/// initializer is placed directly in `.bss` at link time, so obtaining it
/// costs no stack space at all, unlike a `StaticCell::init` call which
/// would momentarily hold the value by-value in whatever called it.
static mut FRAME: Frame = [0u8; BUFFER_SIZE];

/// Sleep for `ms`, feeding the watchdog at least once per <=1 s slice (port
/// of `sys_idle_ms`). `sys_idle_ms` also pumped `cyw43_arch_poll()` every
/// 10 ms to keep the C firmware's cooperative lwIP/cyw43 driver alive while
/// "idle"; that has no equivalent here since embassy's `cyw43_task` and
/// `net_task` are independently scheduled background tasks, woken by
/// interrupts rather than by a manual poll from this loop.
async fn idle_ms(wdt: &mut Watchdog, ms: u32) {
    let mut remaining = ms;
    while remaining > 0 {
        let slice = remaining.min(1000);
        wdt.feed(WDT_RELOAD);
        Timer::after_millis(slice as u64).await;
        remaining -= slice;
    }
}

/// Feed the watchdog while `fut` is pending, so a merely-slow network
/// operation inside its own legitimate budget (`net::join`'s 30 s,
/// `sntp::query`'s 10 s, `net_http::post_json`'s 10+15+30 s DNS/connect/
/// exchange budgets inside `odoo_client::sync`) never looks like a hung
/// executor to the 8 s watchdog — matching the C firmware, which fed every
/// 5-50 ms from inside these same waits (`net_wifi.c`, `http_conn.c`).
///
/// Races `fut` against a 1 s `Timer` in a loop: every tick feeds
/// `WDT_RELOAD` and re-arms the timer; the first time `fut` itself
/// resolves, its output is returned (the still-pending timer is dropped).
/// A genuine executor hang (nothing scheduled at all, this future included)
/// still reboots, since feeding only happens while this future is actually
/// being polled. Each wrapped operation's own `with_timeout` still bounds
/// how long it can run; this only prevents a slow-but-still-within-budget
/// wait from starving the watchdog.
async fn fed<F: core::future::Future>(wdt: &mut Watchdog, fut: F) -> F::Output {
    let mut fut = pin!(fut);
    loop {
        match select(fut.as_mut(), Timer::after_secs(1)).await {
            Either::First(out) => return out,
            Either::Second(()) => wdt.feed(WDT_RELOAD),
        }
    }
}

/// Port of `main.c`'s `wifi_boot`: join with exponential backoff (5 s
/// doubling to an 80 s cap) until connected. Log strings are frozen
/// byte-exact to match the C firmware's serial output. `net::init`'s
/// radio/stack bring-up now happens once in `main`, before this is called
/// (the brief's boot order splits "net init" from "wifi_boot" so the
/// watchdog can be armed and fed across both this join loop and the
/// earlier boot steps).
async fn wifi_boot(
    control: &mut cyw43::Control<'static>,
    stack: Stack<'static>,
    wdt: &mut Watchdog,
) {
    let mut backoff: u32 = 5_000;

    while !fed(wdt, net::join(control, stack)).await {
        log::info!("wifi: retry in {} ms", backoff);
        idle_ms(wdt, backoff).await;
        backoff = if backoff < 80_000 {
            backoff * 2
        } else {
            80_000
        };
    }
    log::info!("wifi: connected");
}

/// Port of `main.c`'s `time_boot`: query SNTP every 500 ms until a reply
/// lands, then anchor the wall clock to it. Log string frozen byte-exact.
async fn time_boot(stack: Stack<'static>, wdt: &mut Watchdog) -> Clock {
    loop {
        if let Some(unix_secs) = fed(wdt, sntp::query(stack)).await {
            log::info!("time: synced");
            return Clock::new(unix_secs);
        }
        idle_ms(wdt, 500).await;
    }
}

/// Port of `note_result`: on success, reset the failure streak and clear
/// `offline`; on failure, restore the last-displayed list into `current`
/// (so a transient fetch failure never blanks the panel's next render),
/// count the failure, and on the 3rd consecutive failure mark offline and
/// stamp `offline_since` with the current local time.
fn note_result(app: &mut App, ok: bool, clock: &Clock) {
    if ok {
        app.fails = 0;
        app.current.offline = false;
        log::info!("odoo: {} activities", app.current.list.count);
        return;
    }
    app.fails += 1;
    app.current.list = app.displayed.list.clone();
    log::info!("odoo: fetch failed ({} in a row)", app.fails);
    if app.fails == 3 {
        app.current.offline = true;
        let (_, hour, minute) = clock.now_local();
        app.offline_since = hhmm(hour, minute);
    }
}

/// Port of `display_frame`: init the panel, push the framebuffer, sleep it.
/// `epd.sleep()` always runs once `init` has succeeded, even if `display`
/// itself fails, matching the C firmware (and the panel's own expectation
/// that a successful power-on is always paired with a sleep command).
fn display_frame(epd: &mut Epd<'static>, fb: &Frame, wdt: &mut Watchdog) -> bool {
    log::info!("epd: refresh start");
    if epd.init(wdt).is_err() {
        log::info!("epd: panel not responding");
        return false;
    }
    let ok = epd.display(fb, wdt).is_ok();
    epd.sleep();
    log::info!("epd: refresh {}", if ok { "done" } else { "failed" });
    ok
}

/// Port of `apply_refresh`: render the current snapshot into the
/// framebuffer and push it to the panel; `displayed`/`last_refresh_s`/
/// `has_displayed` only advance once the panel actually accepted the frame.
fn apply_refresh(
    app: &mut App,
    fb: &mut Frame,
    epd: &mut Epd<'static>,
    wdt: &mut Watchdog,
    clock: &Clock,
) {
    let (today, hour, minute) = clock.now_local();
    let banner_date = banner(today);
    let updated_hhmm = hhmm(hour, minute);
    let data = DashboardData {
        snap: &app.current,
        today,
        banner_date: &banner_date,
        updated_hhmm: &updated_hhmm,
        offline_since: &app.offline_since,
    };
    dashboard::render(fb, &data);
    if !display_frame(epd, fb, wdt) {
        return;
    }
    app.displayed = app.current.clone();
    app.last_refresh_s = clock.uptime_s();
    app.has_displayed = true;
}

/// Port of `poll_once`: if the association dropped, attempt one silent
/// rejoin; sync with Odoo, note the result, and refresh the panel if the
/// refresh policy says the change (or age) warrants it.
#[allow(clippy::too_many_arguments)]
async fn poll_once(
    stack: Stack<'static>,
    rng: &mut embassy_rp::trng::Trng<'static, embassy_rp::peripherals::TRNG>,
    control: &mut cyw43::Control<'static>,
    epd: &mut Epd<'static>,
    wdt: &mut Watchdog,
    clock: &Clock,
    fb: &mut Frame,
    app: &mut App,
) {
    // Port of `poll_once`'s `if (net_wifi_up() == 0) net_wifi_connect();`:
    // one silent rejoin attempt (no new log strings, matching C) when the
    // association has dropped. Result is ignored either way — if it's
    // still down, `odoo_client::sync` below will fail and the existing
    // failure-streak/offline bookkeeping in `note_result` takes over.
    if !net::is_up(stack) {
        let _ = fed(wdt, net::join(control, stack)).await;
    }

    let times = RefreshTimes {
        now_s: clock.uptime_s(),
        last_refresh_s: app.last_refresh_s,
        has_displayed: app.has_displayed,
    };

    let ok = fed(
        wdt,
        odoo_client::sync(stack, rng, &mut app.uid, &mut app.current.list),
    )
    .await;
    note_result(app, ok, clock);
    if !ok && app.fails < 3 {
        return;
    }
    if refresh_needed(&app.displayed, &app.current, &times) {
        apply_refresh(app, fb, epd, wdt, clock);
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    spawner.spawn(logger_task(Driver::new(p.USB, Irqs)).unwrap());

    Timer::after_secs(2).await;
    panic_log::report();

    let mut wdt = Watchdog::new(p.WATCHDOG);
    wdt.start(WDT_RELOAD);

    let wifi_pins = net::WifiPins {
        pwr: p.PIN_23,
        cs: p.PIN_25,
        pio: p.PIO0,
        dio: p.PIN_24,
        clk: p.PIN_29,
        dma: p.DMA_CH0,
    };
    let (stack, mut control) = net::init(spawner, wifi_pins).await;

    let mut epd = Epd::new(p.PIN_12, p.PIN_8, p.PIN_9, p.PIN_10, p.PIN_11, p.PIN_13);

    wifi_boot(&mut control, stack, &mut wdt).await;
    let clock = time_boot(stack, &mut wdt).await;

    let mut rng = net_http::new_rng(p.TRNG);

    // SAFETY: taken once here, before the superloop starts, and never
    // aliased again (no other code ever accesses FRAME).
    let fb: &'static mut Frame = unsafe { &mut *addr_of_mut!(FRAME) };

    static APP: StaticCell<App> = StaticCell::new();
    let app = APP.init(App::default());

    loop {
        poll_once(
            stack,
            &mut rng,
            &mut control,
            &mut epd,
            &mut wdt,
            &clock,
            fb,
            app,
        )
        .await;
        idle_ms(&mut wdt, POLL_INTERVAL_S * 1000).await;
    }
}
