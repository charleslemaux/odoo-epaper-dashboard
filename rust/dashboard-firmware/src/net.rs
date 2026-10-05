//! cyw43 WiFi bring-up (port of `src/net/net_wifi.c`).
//!
//! Brings the CYW43439 radio up over the Pico 2 W's PIO-driven half-duplex
//! SPI bus, spawns the cyw43 and embassy-net background runners, and joins
//! the configured access point with a DHCP lease. Unlike the C driver
//! (`cyw43_arch_init_with_country` + polling `cyw43_arch_wifi_connect_async`
//! in a loop), the embassy stack is fully async: `init` brings the chip up
//! once at boot, and `join` is called repeatedly from `main`'s boot/retry
//! loop (and later from the superloop's reconnect check).
use cyw43::{Aligned, Control, JoinOptions, NetDriver, PowerManagementMode, SpiBus, State, A4};
use cyw43_pio::{PioSpi, DEFAULT_CLOCK_DIVIDER};
use embassy_executor::Spawner;
use embassy_net::{Config, Stack, StackResources};
use embassy_rp::bind_interrupts;
use embassy_rp::clocks::RoscRng;
use embassy_rp::dma::{Channel as DmaChannel, InterruptHandler as DmaInterruptHandler};
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{DMA_CH0, PIN_23, PIN_24, PIN_25, PIN_29, PIO0};
use embassy_rp::pio::{InterruptHandler as PioInterruptHandler, Pio};
use embassy_rp::Peri;
use embassy_time::{with_timeout, Duration};
use static_cell::StaticCell;

use crate::config::{WIFI_PASSWORD, WIFI_SSID};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
    DMA_IRQ_0 => DmaInterruptHandler<DMA_CH0>;
});

/// Number of sockets reserved in the embassy-net stack: one for DHCP, one
/// for DNS, and the rest free for the application (SNTP's UDP socket now,
/// the odoo client's TCP+TLS socket in T17).
const SOCKET_COUNT: usize = 8;

/// Join timeout, mirroring C's `net_wifi_connect`'s 30 s deadline.
const JOIN_TIMEOUT: Duration = Duration::from_secs(30);

static FIRMWARE: Aligned<A4, [u8; include_bytes!("../cyw43-firmware/43439A0.bin").len()]> =
    Aligned(*include_bytes!("../cyw43-firmware/43439A0.bin"));
static CLM: Aligned<A4, [u8; include_bytes!("../cyw43-firmware/43439A0_clm.bin").len()]> =
    Aligned(*include_bytes!("../cyw43-firmware/43439A0_clm.bin"));

type Cyw43Bus = SpiBus<Output<'static>, PioSpi<'static, PIO0, 0>>;

#[embassy_executor::task]
async fn cyw43_task(runner: cyw43::Runner<'static, Cyw43Bus>) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, NetDriver<'static>>) -> ! {
    runner.run().await
}

/// Pins wired to the CYW43439 on a standard Pico 2 W.
pub struct WifiPins {
    pub pwr: Peri<'static, PIN_23>,
    pub cs: Peri<'static, PIN_25>,
    pub pio: Peri<'static, PIO0>,
    pub dio: Peri<'static, PIN_24>,
    pub clk: Peri<'static, PIN_29>,
    pub dma: Peri<'static, DMA_CH0>,
}

/// Bring the radio up: load firmware/CLM, spawn the cyw43 and embassy-net
/// background runners, and bring up DHCP. Does not join a network yet.
pub async fn init(spawner: Spawner, pins: WifiPins) -> (Stack<'static>, Control<'static>) {
    static STATE: StaticCell<State> = StaticCell::new();
    let state = STATE.init(State::new());

    let pwr = Output::new(pins.pwr, Level::Low);
    let cs = Output::new(pins.cs, Level::High);
    let mut pio = Pio::new(pins.pio, Irqs);
    let dma = DmaChannel::new(pins.dma, Irqs);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        DEFAULT_CLOCK_DIVIDER,
        pio.irq0,
        cs,
        pins.dio,
        pins.clk,
        dma,
    );

    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, &FIRMWARE, &CLM).await;
    spawner.spawn(cyw43_task(runner).unwrap());

    control.init(&*CLM).await;
    control
        .set_power_management(PowerManagementMode::PowerSave)
        .await;

    let config = Config::dhcpv4(Default::default());
    let seed = RoscRng.next_u64();

    static RESOURCES: StaticCell<StackResources<SOCKET_COUNT>> = StaticCell::new();
    let resources = RESOURCES.init(StackResources::new());
    let (stack, runner) = embassy_net::new(net_device, config, resources, seed);
    spawner.spawn(net_task(runner).unwrap());

    (stack, control)
}

/// Join the configured access point and wait for a DHCP lease, 30 s total.
/// Returns `true` once `stack.wait_config_up()` resolves within the
/// deadline; `false` on any join failure or timeout (port of
/// `net_wifi_connect`).
pub async fn join(control: &mut Control<'static>, stack: Stack<'static>) -> bool {
    let attempt = async {
        control
            .join(WIFI_SSID, JoinOptions::new(WIFI_PASSWORD.as_bytes()))
            .await
            .ok()?;
        stack.wait_config_up().await;
        Some(())
    };
    matches!(with_timeout(JOIN_TIMEOUT, attempt).await, Ok(Some(())))
}
