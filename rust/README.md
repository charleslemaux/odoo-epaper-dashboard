# Rust rewrite — dashboard-firmware (Embassy, RP2350)

This is an in-progress Rust port of the C firmware in the repo root,
built on [Embassy](https://embassy.dev) for the Pico 2 W (RP2350 +
CYW43439). It lives entirely under `rust/` on branch
`feat/rust-rewrite` and does not touch the C build. The hardware,
wiring, panel behavior, and Odoo integration are all unchanged from
the root `README.md` — this document only covers what's different
about building, testing, and flashing the Rust firmware.

Development proceeds in small, gated tasks (see
`.superpowers/sdd/2026-10-05-rust-rewrite/`); each task's report
records exactly which commands were run and what they produced. This
README distills that history into one reference, kept in sync with
what actually works, not with the original plan.

## 1. Toolchain setup

Install `rustup` (PowerShell, user-scope, no admin needed):

```powershell
winget install -e --id Rustlang.Rustup --accept-source-agreements --accept-package-agreements
```

(The correct winget id is `Rustlang.Rustup` — a common typo is
`Rustlan.Rustup`, which does not exist.) `rustup-init`'s default
install already pulls the `stable` toolchain; confirm and add the
embedded target + components explicitly once:

```powershell
rustup default stable
rustup target add thumbv8m.main-none-eabihf
rustup component add clippy rustfmt
```

`rust/rust-toolchain.toml` pins `channel = "stable"`, the
`thumbv8m.main-none-eabihf` target, and the `clippy`/`rustfmt`
components, so any `cargo` invocation from inside `rust/` auto-selects
and (if needed) auto-installs this exact toolchain — the explicit
`rustup` commands above are only needed once, the first time.

Versions this was actually built and gated against:

```
rustup 1.29.1
rustc 1.99.0 (stable)
cargo 1.99.0 (stable)
```

PowerShell environment preamble used before every command below (this
machine's PATH does not otherwise pick up Cargo's shims or the LLVM
tools `rust-lld` needs in every fresh shell):

```powershell
$env:Path = [Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [Environment]::GetEnvironmentVariable('Path','User') + ';' + "$env:USERPROFILE\.cargo\bin" + ';C:\Program Files\Git\usr\bin;C:\Program Files\LLVM\bin'
```

**Smart App Control**: this project's dev machine has Windows Smart
App Control enabled (see the root README's section 2 for the
C-toolchain side of this). It did not block any freshly-compiled Rust
binary during this port (`cargo run`/`cargo test`/`picotool.exe` all
ran clean on first try across every task). If a fresh binary is ever
blocked ("Access is denied" running a just-built `.exe`), the
established workaround for this repo is: delete the binary, rebuild,
and retry — SAC reliably admits a rebuilt binary it previously
blocked.

**`picotool`**: any picotool 2.x works; this machine uses a prebuilt
install at `C:\Users\charl\picotool-install\bin\picotool.exe`
(`picotool v2.3.0`, built without USB support — the `uf2 convert`
subcommand used below doesn't need USB).

## 2. Workspace layout and why `cargo test`/`cargo clippy` never use `--workspace`

`rust/Cargo.toml`:

```toml
[workspace]
members = ["dashboard-core", "host-tools", "dashboard-firmware"]
default-members = ["dashboard-core", "host-tools"]
```

- `dashboard-core` — `no_std`-capable pure logic (JSON parsing, Odoo
  response parsing, graphics/text/dashboard rendering, refresh policy,
  PEM decoding), unit-tested on the host.
- `host-tools` — host-only binaries/tests: the `preview` tool and the
  C/Rust frame-parity test (section 4).
- `dashboard-firmware` — the actual `no_std`/`no_main` RP2350 binary.
  **Deliberately excluded from `default-members`.** It targets
  `thumbv8m.main-none-eabihf` — `cortex-m`'s inline `asm!` (`sev`,
  `msplim`, ...) does not exist on the host `x86_64` target at all, so
  compiling it for the host is not merely "wrong," it's a hard
  compile error:
  ```
  error: invalid instruction mnemonic 'sev'
  error[E0433]: cannot find `msplim` in `register`
  ```
  (`cargo test --workspace`/`cargo clippy --workspace` explicitly
  override `default-members` — see Cargo's own docs — so passing
  either flag always tries to build `dashboard-firmware` for the host
  and always fails this way. **Never pass `--workspace` in this
  repo.**)

The gates below are therefore always run as **bare** `cargo`
invocations (no `-p`, no `--workspace`) from `rust/` — a bare
invocation scopes to `default-members`, i.e. host-buildable code only
— plus separate, explicit `-p dashboard-firmware --target
thumbv8m.main-none-eabihf` invocations for the firmware itself.
`cargo fmt --check` is the one exception: it is scope-independent and
always formats every workspace member, firmware included.

## 3. Configuration

Copy the template and fill in real values — `src/config.rs` is
gitignored from the first commit, same reasoning as the C firmware's
`include/config.h`:

```powershell
cp rust/dashboard-firmware/src/config.rs.example rust/dashboard-firmware/src/config.rs
```

Every constant, in the order they appear in `config.rs.example`:

| Constant | Type | Meaning |
|---|---|---|
| `WIFI_SSID` / `WIFI_PASSWORD` | `&str` | Your WiFi network credentials. |
| `WIFI_COUNTRY` | `&str` | **Currently inert.** `cyw43 0.7`'s `Control::init` hardcodes the regulatory domain to `countries::WORLD_WIDE_XX` internally and exposes no parameter to override it (unlike the C firmware's `cyw43_arch_init_with_country(WIFI_COUNTRY)`). The constant is kept — and still consumed by `config.rs`'s own `#![allow(dead_code)]` — purely for forward-compatibility with a future `cyw43` release that re-exposes per-country config. In practice this only matters for networks that require channels 12/13 (a non-worldwide regulatory domain); ordinary 1-11 channel APs join identically either way. |
| `ODOO_HOST` | `&str` | Odoo hostname only — no `https://`, no path. |
| `ODOO_PORT` | `u16` | HTTPS port, normally `443`. |
| `ODOO_DB` | `&str` | Odoo database name. |
| `ODOO_LOGIN` | `&str` | Your Odoo login (usually an email address). |
| `ODOO_API_KEY` | `&str` | An Odoo API key (not your account password) — see the root README's "Creating an Odoo API key". |
| `ODOO_ACTIVITY_DOMAIN` | `&str` | A JSON-RPC domain filter on `mail.activity`, with `%d` where the authenticated `uid` goes — same semantics and default as the C firmware's macro of the same name; see the root README's "Tuning `ODOO_ACTIVITY_DOMAIN`". |
| `POLL_INTERVAL_S` | `u32` | Seconds between Odoo polls, default `300` (5 minutes). |
| `NTP_SERVER` | `&str` | SNTP server for time sync, e.g. `pool.ntp.org`. |
| `TZ_OFFSET_MIN` | `i32` | Local timezone offset from UTC, in minutes (no DST database — a fixed offset, same as the C firmware). |
| `EPD_CLK_HALF_PERIOD_US` | `u32` | Bit-bang clock half-period in microseconds. Default `1`. Do not lower without re-validating on real hardware — same SPI-corruption caveat as the C firmware (root README section 1). |
| `ODOO_CA_CERT` | `Option<&str>` | **`None`** (default): TLS still encrypts the connection but does not verify the server's identity (same unverified-by-default posture as the C firmware). **`Some(pem)`**: enables real certificate-chain-of-trust, signature, and hostname (SNI) verification via `embedded-tls`'s `rustpki` verifier — a genuine `no_std` X.509 path, not a stub. **Caveat: certificate expiry is not checked.** The verifier takes a `TlsClock` type parameter for the not-before/not-after window check; this firmware passes `NoClock` (no wall-clock plumbed into that call), and `embedded-tls` is documented to skip the expiry check gracefully (not fail) when the clock returns `None`. So an expired or not-yet-valid pinned cert is still accepted, but a cert signed by the wrong CA, or presented for the wrong hostname, is correctly rejected. If the configured PEM fails to parse, the firmware logs `tls: ODOO_CA_CERT is not valid PEM; proceeding WITHOUT certificate verification` and falls back to the unverified path rather than failing every request. |

To pin a CA certificate, fetch it the same way as the root README's
"Optional: pinning the Odoo CA certificate" section:

```
openssl s_client -connect ODOO_HOST:443 -showcerts
```

Take the last `-----BEGIN CERTIFICATE-----`/`-----END CERTIFICATE-----`
block (the issuing CA), and set it as a Rust string literal (raw
strings make the embedded newlines easy):

```rust
pub const ODOO_CA_CERT: Option<&str> = Some(r#"-----BEGIN CERTIFICATE-----
MIIB...
-----END CERTIFICATE-----
"#);
```

**Keep config values free of anything that would break a `&str`
literal** (an unescaped `"` outside a raw string breaks the build).

## 4. Host tests (gates)

From `rust/`, with the PATH preamble from section 1:

```powershell
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

All three are bare invocations (see section 2) — they build and test
`dashboard-core` and `host-tools` only, never `dashboard-firmware`.

### Parity test

`host-tools/tests/parity.rs` renders the exact scenario from the C
firmware's `tests/render_preview.c` (6 activities, total 11, today
2026-08-27, banner `jeu 27/08`, updated `03:15`) through
`dashboard_core::dashboard::render` and asserts the resulting 192,000-byte
framebuffer is **byte-for-byte identical** to
`host-tools/tests/fixtures/preview_c.raw`, a fixture generated once
from the actual C renderer and committed as ground truth. On a
mismatch, the panic message localizes the first divergent byte to an
approximate pixel `(x, y)` to make debugging tractable.

**The fixture is ground truth — do not regenerate it to make the test
pass.** A failing parity test means the Rust port diverged from the C
renderer; fix the Rust side. The fixture is only ever regenerated when
the layout is *intentionally* changed **in both implementations at
once** (e.g. a deliberate redesign of the dashboard layout that
updates both `src/app/dashboard.c` and
`dashboard-core/src/dashboard.rs` together). When that's genuinely the
case, from the repo root (PowerShell, usual preamble):

```powershell
make -C tests CC=clang preview.exe
Set-Location tests; .\preview.exe; Set-Location ..
Copy-Item tests\preview.raw rust\host-tools\tests\fixtures\preview_c.raw
(Get-Item rust\host-tools\tests\fixtures\preview_c.raw).Length   # must print 192000
```

(`CC=clang` matters on this machine specifically — the default
WinLibs GCC substitute's assembler is blocked by Smart App Control;
see the root README's section 2/6 for the full story.) Re-run `cargo
test` afterward to confirm the newly-regenerated fixture matches the
newly-changed Rust renderer.

An optional visual check: `cargo run -p host-tools --bin preview`
writes `preview.raw` in the current directory, viewable with the same
raw→PNG conversion used for the C workflow.

## 5. Firmware build and flash

```powershell
cargo clippy -p dashboard-firmware --target thumbv8m.main-none-eabihf -- -D warnings
cargo build --release -p dashboard-firmware --target thumbv8m.main-none-eabihf
```

Both run from `rust/`, same PATH preamble. The release build has one
benign, non-blocking linker warning on every build (confirmed
pre-existing at the very first firmware scaffold, reproduced
byte-identically upstream in embassy-rs/embassy's own `rp235x`
example `memory.x`, not gated by `-D warnings`):

```
rust-lld: address (0x1000013c) of section .text is not a multiple of alignment (8)
```

Convert the linked ELF to a flashable UF2 (any picotool 2.x):

```powershell
& "C:\Users\charl\picotool-install\bin\picotool.exe" uf2 convert `
    target\thumbv8m.main-none-eabihf\release\dashboard-firmware `
    -t elf dashboard-firmware.uf2
```

To flash: hold **BOOTSEL**, plug the Pico 2 W into USB, release BOOTSEL
once it shows up as a mass-storage drive (`RP2350`), then drag-and-drop
`dashboard-firmware.uf2` onto that drive — same procedure as the C
firmware's `build/epaper_dashboard.uf2`. The board reboots and starts
running automatically.

Open a serial monitor at **115200 baud** on the Pico's USB serial port
(the firmware uses `embassy-usb-logger` over the same USB CDC-ACM
interface, not the SWD/debug UART) to watch the boot log — see section
6 below for exactly what to expect.

The CYW43439 WiFi firmware blobs the radio driver needs
(`43439A0.bin`, `43439A0_clm.bin`) are vendored under
`dashboard-firmware/cyw43-firmware/`, fetched from the
embassy-rs/embassy upstream repository and embedded at compile time
via `include_bytes!` — no separate flashing step for them. They carry
Infineon/Cypress's Permissive Binary License 1.0; see
`dashboard-firmware/cyw43-firmware/LICENSE-permissive-binary-license-1.0.txt`.

## 6. Log expectations

Every log line is frozen byte-exact against the C firmware's serial
output (same wording, same call sites, verified line-by-line across
each porting task). A clean boot and first successful poll looks like:

```
wifi: retry in 5000 ms        (only if the first join attempt fails; repeats with doubling backoff, 5 s -> 80 s cap, until connected)
wifi: connected
time: synced
odoo: authenticated uid=7
odoo: 11 activities
epd: refresh start
epd: refresh done
```

Other lines that can appear in place of the above, depending on
conditions:

- `odoo: fetch failed (N in a row)` — a poll failed; after the 3rd
  consecutive failure the next refresh shows the offline footer.
- `epd: panel not responding` — the panel failed to come out of reset
  during `init`; `epd: refresh done`/`failed` is skipped for that
  cycle.
- `epd: refresh failed` — `init` succeeded but `display` did not; the
  panel is still put to sleep either way.
- `tls: ODOO_CA_CERT is not valid PEM; proceeding WITHOUT certificate
  verification` — only when `ODOO_CA_CERT` is `Some(...)` but fails to
  parse (section 3).
- `last panic: <message>` — logged once, immediately after boot
  (before the first `wifi: retry`/`wifi: connected` line), **only on
  the reboot that follows a panic**. The panic handler stores the
  message in an uninitialized RAM section that survives a watchdog
  reset, then forces an immediate watchdog reset; the next boot's
  `panic_log::report()` call reads it, logs it exactly once, and
  clears it so it is never reported twice.

## 7. Crate choices, versions, and the pinning/reconciliation policy

The original design plan pinned specific crate versions at the time it
was written. By the time each task actually ran, ecosystem drift meant
several of those pins no longer existed on crates.io (entire minor
lines superseded) or had moved to incompatible transitive dependencies.
The policy applied throughout this port, and the one to keep following:

1. Try the plan's pinned version first.
2. If it's gone or conflicts with an already-resolved sibling
   dependency, resolve to the latest crates.io version that is
   lockstep-compatible with the embassy-family crates already pinned
   (`embassy-executor`/`embassy-time`/`embassy-rp` move together; so do
   `cyw43`/`cyw43-pio`/`embassy-net`'s `embedded-io-async` requirement).
3. Verify the replacement against the crate's actual vendored source
   under `~/.cargo/registry/src/...` (not just its doc comments) before
   relying on an API shape.
4. Record every such deviation in that task's report, with the
   reasoning — never silently reconcile.

Crates actually in use today (`dashboard-firmware/Cargo.toml`):

| Crate | Version | Role |
|---|---|---|
| `embassy-executor` | 0.10 | async executor (`platform-cortex-m`, `executor-thread`) |
| `embassy-time` | 0.5 | timers, delays |
| `embassy-rp` | 0.10 | RP2350 HAL (GPIO, watchdog, TRNG, DMA, PIO) |
| `embassy-net` | 0.9 | DHCP/UDP/TCP/DNS network stack (smoltcp-backed) |
| `embassy-futures` | 0.1 | `select`/`Either`, used by the watchdog-feeding `fed` combinator |
| `cyw43` | 0.7 | CYW43439 WiFi driver |
| `cyw43-pio` | 0.10 | PIO-backed SPI bus for `cyw43` |
| `embedded-tls` | 0.19 (`default-features = false`, `features = ["rustpki"]`) | TLS 1.3 client, `no_std` CA-pinning verifier |
| `embassy-usb-logger` | 0.6 | `log`-over-USB-CDC serial output |
| `heapless` | 0.8 (workspace-pinned) | fixed-capacity `String`/`Vec` (coexists with `cyw43`'s own `heapless 0.9` pull-in; no conflict, nothing crosses that boundary) |
| `static_cell` | 2 | one-shot statics for the network stack/state and the top-level `App` |
| `log` | 0.4 | logging facade |

A few concrete reconciliations worth knowing about if touching this
code (full detail in each task's report under
`.superpowers/sdd/2026-10-05-rust-rewrite/`):

- `embassy-executor`/`embassy-time`/`embassy-rp`/`embassy-usb(-logger)`
  all moved from their originally-pinned `0.4`/`0.7` lines to
  `0.10`/`0.5`/`0.10`/`0.6` — those older lines simply don't exist on
  crates.io anymore.
- `embedded-tls` moved from the plan's `0.17` to `0.19`: `0.17` needs
  `embedded-io-async ^0.6`, but `embassy-net 0.9.1`'s `TcpSocket`
  implements the `^0.7` `Read`/`Write` traits — a different, mutually
  exclusive trait identity, not just "two versions coexisting." `0.18`
  and `0.19` both moved to `^0.7`; `0.19` was picked as the newer of
  the two compatible releases.
- `WIFI_COUNTRY` being inert (section 3) was discovered by reading
  `cyw43 0.7.0`'s actual source, not assumed from its docs.

## 8. Known behavior differences from the C firmware

- **`WIFI_COUNTRY` is inert** — see section 3. Low risk (affects only
  channel 12/13 regulatory behavior), flagged for whoever upgrades
  `cyw43` past 0.7 in case a country-override API reappears.
- **Pinned-CA TLS does not check certificate expiry** — see section 3.
  Chain-of-trust, signature, and hostname are still fully verified.
- **No wifi reconnect inside `idle_ms`/`poll_once`'s own waits beyond
  one rejoin attempt per poll** — if the CYW43 association itself
  drops, `poll_once` attempts exactly one silent rejoin per poll cycle
  before falling through to the normal fetch (which will then fail and
  drive the existing offline-footer bookkeeping) rather than retrying
  in a tight loop.

These are also the kinds of things [HW] end-to-end validation (flashing
real hardware with a real `config.rs` and watching the serial log
against section 6's expectations) is there to catch if any of them
turn out to matter in practice.
