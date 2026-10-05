//! e-paper dashboard firmware (Embassy, RP2350).
#![no_std]
#![no_main]

mod config;
mod epd;

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

    loop {
        log::info!("epaper_dashboard: toolchain OK");
        Timer::after_secs(1).await;
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
