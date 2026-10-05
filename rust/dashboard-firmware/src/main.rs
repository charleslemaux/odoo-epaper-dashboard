//! e-paper dashboard firmware (Embassy, RP2350).
#![no_std]
#![no_main]

mod clock;
mod config;
mod epd;
mod net;
mod sntp;

use embassy_executor::Spawner;
use embassy_rp::block::ImageDef;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_rp::{bind_interrupts, peripherals::USB};
use embassy_time::Timer;

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

/// Port of `main.c`'s `wifi_boot`: bring the radio up, then join with
/// exponential backoff (5 s doubling to an 80 s cap) until connected.
/// Log strings are frozen byte-exact to match the C firmware's serial
/// output.
async fn wifi_boot(
    spawner: Spawner,
    pins: net::WifiPins,
) -> (embassy_net::Stack<'static>, cyw43::Control<'static>) {
    let (stack, mut control) = net::init(spawner, pins).await;
    let mut backoff: u32 = 5_000;

    while !net::join(&mut control, stack).await {
        log::info!("wifi: retry in {} ms", backoff);
        Timer::after_millis(backoff as u64).await;
        backoff = if backoff < 80_000 {
            backoff * 2
        } else {
            80_000
        };
    }
    log::info!("wifi: connected");
    (stack, control)
}

/// Port of `main.c`'s `time_boot`: query SNTP every 500 ms until a reply
/// lands, then anchor the wall clock to it. Log string frozen byte-exact.
async fn time_boot(stack: embassy_net::Stack<'static>) -> clock::Clock {
    loop {
        if let Some(unix_secs) = sntp::query(stack).await {
            log::info!("time: synced");
            return clock::Clock::new(unix_secs);
        }
        Timer::after_millis(500).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    spawner.spawn(logger_task(Driver::new(p.USB, Irqs)).unwrap());

    // Smoke-wiring only: construct the driver from its pins to prove the
    // module compiles and the pin assignment is wired correctly. Panel
    // safety: init()/display() are NOT called here (no hardware refresh
    // cycle from the toolchain-check skeleton).
    let mut epd = epd::Epd::new(p.PIN_12, p.PIN_8, p.PIN_9, p.PIN_10, p.PIN_11, p.PIN_13);
    let _ = &mut epd;

    let wifi_pins = net::WifiPins {
        pwr: p.PIN_23,
        cs: p.PIN_25,
        pio: p.PIO0,
        dio: p.PIN_24,
        clk: p.PIN_29,
        dma: p.DMA_CH0,
    };
    let (stack, control) = wifi_boot(spawner, wifi_pins).await;
    let clock = time_boot(stack).await;
    // Not yet consumed past boot: the odoo/TLS client (T17) and the
    // dashboard poll loop (T18) are the next tasks to drive these further.
    let _ = (&control, &clock);

    loop {
        log::info!("epaper_dashboard: toolchain OK");
        Timer::after_secs(1).await;
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
