//! Panic message capture surviving reboot for headless diagnostics
//! (port of `src/app/panic_log.c`).
//!
//! The panic message is written into a `static` placed in the `.uninit`
//! section that `cortex-m-rt`'s linker script collects via `*(.uninit
//! .uninit.*)` (NOLOAD, never zero-initialized by the runtime), the same
//! trick the C firmware gets from the pico-sdk's `__uninitialized_ram`: the
//! bytes survive a watchdog reset because the reset never power-cycles RAM.
//!
//! Access to the `static mut` below always goes through `addr_of_mut!` plus
//! raw-pointer reads/writes, never a `&`/`&mut` borrow of the static
//! itself, to steer clear of the `static_mut_refs` lint (denied by
//! `-D warnings`) while still being sound: both `report()` (boot, before
//! the executor and any other task exists) and the `#[panic_handler]`
//! (which runs in place of whatever faulted, with nothing else executing)
//! only ever touch this memory one call at a time.

use core::fmt::Write;
use core::ptr::addr_of_mut;

use dashboard_core::bufwrite::SliceWriter;
use embassy_rp::peripherals::WATCHDOG;
use embassy_rp::watchdog::Watchdog;

const PANIC_LOG_MAGIC: u32 = 0x50414e43;
const PANIC_LOG_CAP: usize = 160;

#[repr(C)]
struct PanicLog {
    magic: u32,
    message: [u8; PANIC_LOG_CAP],
}

#[link_section = ".uninit.PANIC"]
static mut PANIC_LOG: PanicLog = PanicLog {
    magic: 0,
    message: [0; PANIC_LOG_CAP],
};

/// Port of `panic_log_report`: if the previous boot left a panic message
/// behind, log it once (`last panic: {msg}`) and clear the magic so it is
/// never reported twice. Call once at boot, before anything else touches
/// `PANIC_LOG`.
pub fn report() {
    // SAFETY: called once from `main`, before the executor and any task
    // (including the panic handler, which cannot run until something is
    // executing) exist, so this is the only live access to PANIC_LOG.
    let magic = unsafe { addr_of_mut!(PANIC_LOG.magic).read() };
    if magic != PANIC_LOG_MAGIC {
        return;
    }
    // SAFETY: same single-access-at-a-time argument as above.
    let message = unsafe { addr_of_mut!(PANIC_LOG.message).read() };
    let len = message
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(PANIC_LOG_CAP);
    let msg = core::str::from_utf8(&message[..len]).unwrap_or("(invalid panic message)");
    log::info!("last panic: {}", msg);
    // SAFETY: same single-access-at-a-time argument as above.
    unsafe { addr_of_mut!(PANIC_LOG.magic).write(0) };
}

/// Replaces the toolchain-check skeleton's `loop {}` handler. Formats the
/// panic message into `PANIC_LOG`, sets the magic, then reboots via the
/// watchdog's immediate forced-reset trigger (port of `panic_log_store`,
/// which stores the message, sets the magic, and calls
/// `watchdog_reboot(0, 0, 200)`).
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // Reserve the final byte so the stored message is always
    // NUL-terminated, mirroring `vsnprintf`'s truncation behavior; a
    // message that doesn't fit is silently cut short rather than failing
    // to record anything.
    let mut buf = [0u8; PANIC_LOG_CAP];
    {
        let mut w = SliceWriter::new(&mut buf[..PANIC_LOG_CAP - 1]);
        let _ = write!(w, "{}", info.message());
    }
    // SAFETY: a panic handler runs instead of whatever faulted; nothing
    // else is executing concurrently, so this is the only live access to
    // PANIC_LOG. Message is written before the magic, so a reader only
    // ever observes the magic set once the message is complete.
    unsafe {
        addr_of_mut!(PANIC_LOG.message).write(buf);
        addr_of_mut!(PANIC_LOG.magic).write(PANIC_LOG_MAGIC);
    }

    // SAFETY: stealing the WATCHDOG peripheral here is sound: the panic
    // handler never returns, so this freshly-constructed handle is never
    // used concurrently with the one owned by `main` (which itself will
    // never run again), and `trigger_reset` only performs register writes.
    let mut wdt = Watchdog::new(unsafe { WATCHDOG::steal() });
    wdt.trigger_reset();
    loop {
        core::hint::spin_loop();
    }
}
