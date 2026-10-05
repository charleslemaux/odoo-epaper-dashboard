# Rust Rewrite of the e-Paper Dashboard Firmware — Design

Date: 2026-10-05
Status: approved design, pending implementation plan
Branch: feat/rust-rewrite

## 1. Goal

Rewrite the C firmware in Rust with **behavioral 1:1 parity** (Phase 1),
then refactor into idiomatic Rust and measure efficiency gains
(Phase 2). At the end of the branch, Rust **replaces** the C tree and
becomes the firmware; until then the C code stays in-tree because the
parity harness diffs against it.

"Behavioral 1:1" means: same boot sequence and serial log strings, same
5-minute polling cadence, same refresh-on-change policy (180 s spacing,
24 h health refresh, offline after 3 failures), same Odoo JSON-RPC
request bytes, and **byte-identical rendered framebuffers** verified
against the C renderer. Internal structure follows the Rust ecosystem
(async Embassy) rather than mimicking the C superloop — a structural
1:1 is impossible anyway (no lwIP/mbedTLS exists for bare-metal Rust;
the only maintained CYW43439 WiFi path is Embassy's async `cyw43`).

Verified preconditions (2026-10-05):
- The user's Odoo server negotiates TLS 1.3
  (`TLS_AES_128_GCM_SHA256`) — required because `embedded-tls` is
  TLS 1.3-only.
- No Rust toolchain on the machine yet — bootstrap is part of the plan.

## 2. Approach (decided)

**Faithful port on a minimal Embassy base.** Third-party surface is
limited to what the Pico SDK provided in C: executor/HAL
(`embassy-rp`, `embassy-executor`, `embassy-time`), WiFi driver
(`cyw43` + vendored firmware blobs), TCP/IP (`embassy-net`), TLS
(`embedded-tls`). Everything we wrote ourselves in C is ported 1:1:
JSON tokenizer + UTF-8→ASCII folding, HTTP/1.0 client, Odoo request
builders/parsers, 4bpp gfx + bitmap fonts/icons, refresh policy, time
formatting, dashboard layout, paced bit-bang panel driver, panic log.
Crate-native alternatives (reqwless, serde-json-core,
embedded-graphics) were rejected for Phase 1 (behavioral drift risk);
they may be *evaluated* in Phase 2 only where measurably better.

## 3. Workspace architecture

Cargo workspace at `rust/` on the branch (C tree untouched until the
final promotion commit):

```
rust/
├── Cargo.toml                workspace root
├── dashboard-core/           #![no_std], zero-alloc, no HW deps
│   └── src/: gfx, text/icons render, generated font/icon data,
│        json (tokenizer + folding), odoo (request/parse/model),
│        refresh policy, time_fmt, dashboard layout
├── dashboard-firmware/       Embassy binary, thumbv8m.main-none-eabihf
│   └── src/: main task (boot + poll loop), net task (cyw43),
│        sntp, tls http client, epd driver (paced bit-bang),
│        watchdog, panic log (noinit RAM), config.rs (gitignored)
│        + config.rs.example (committed; same setting names as C)
└── host-tools/               std: preview renderer (sample frame →
     preview.raw → PNG) and parity fixtures
```

`dashboard-core` compiles for both target and host — all ported tests
and the frame-parity test run with plain `cargo test`.

**Task model:** one main Embassy task preserves the C phase sequence
(wifi → SNTP → auth → poll every `POLL_INTERVAL_S` → refresh decision →
display), plus the cyw43/net runner task. Watchdog fed with the same
discipline (during waits, long sends, and refreshes).

**Toolchain:** rustup stable + target `thumbv8m.main-none-eabihf`;
ELF → UF2 via the existing prebuilt `picotool uf2 convert`; flashing
unchanged (BOOTSEL / VS Code). Style gates on every task:
`cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`
(the Rust equivalent of the Epitech audit; Epitech C v7 continues to
govern only the C tree while it exists).

**Generators:** `tools/gen_font.py` and `tools/gen_icons.py` gain a
Rust emitter writing `font_data.rs` / `icon_data.rs` with the same
glyph bytes as the C arrays; a test asserts byte equality so glyph
parity is proven, not assumed.

## 4. Network, protocol, time

- **WiFi/IP:** `cyw43` (same country/SSID/password config) +
  `embassy-net` DHCP/DNS. Reconnect backoff ported (5 s → 80 s cap).
- **TLS:** `embedded-tls`, TLS 1.3. Default posture matches C:
  encrypted, unverified; optional pinned-CA mode remains one config
  entry. Documented behavioral delta: handshake negotiates 1.3 instead
  of 1.2 (invisible to the dashboard).
- **HTTP:** one-shot HTTP/1.0 POST with `Connection: close` per
  request, 16 KB response buffer, same status/body splitting, same
  timeouts (DNS 10 s, connect 15 s, response 30 s — single 30 s
  connect+response budget acceptable as in C).
- **Odoo:** request bodies byte-identical to C (auth, `search_read`
  on `mail.activity` with fields res_name/summary/icon/date_deadline,
  limit 7, order date_deadline asc; `search_count` on the same
  domain). Parser ports 1:1 including `summary: false` → res_name
  fallback and `\uXXXX`/UTF-8 folding. Re-auth-once on Odoo error.
- **Time:** hand-rolled SNTP (one 48-byte UDP exchange) replaces lwIP
  SNTP; wall clock = SNTP epoch anchored to Embassy's monotonic clock
  (replaces the AON timer); same `TZ_OFFSET_MIN`; refresh arithmetic
  driven by monotonic uptime exactly like C's `now_s()`.

**RAM budget (520 KB):** framebuffer 187.5 KB static + embassy-net
~16 KB + embedded-tls record buffers ~33 KB + HTTP/request ~20 KB +
cyw43 ~10 KB + stacks ≈ **~270 KB peak**, fully heapless (no
allocator at all — the mbedTLS handshake heap spike disappears).

**Errors:** same observable policy (silent single failure, offline
footer after 3, watchdog on hang, panic → noinit RAM → `last panic:`
on next boot); internally `Result` types instead of int codes.

## 5. Display and rendering

Panel driver is a direct port: same pins, same 13-command init
sequence, same BUSY semantics/timeouts, same pacing
(`EPD_CLK_HALF_PERIOD_US = 1`, ~500 kHz) via busy-waits, `epd_sleep()`
on every path, ≥3 min spacing + daily refresh invariants, and the
standing prohibition on hardware SPI without on-device revalidation.
Rendering ports with the same layout constants (72 px banner, 7×48 px
rows, icons, red-overdue dates, day chips, counted overflow footer) and
identical font/icon bytes → identical framebuffers.

## 6. Verification

1. **Unit parity:** all 11 C test suites ported to `cargo test` with
   the same assertions; the C suite keeps passing untouched.
2. **Frame parity:** the C preview sample frame is committed as a
   fixture (`preview.raw` from the C tool); a cargo test renders the
   identical scenario via `dashboard-core` and asserts equality on all
   192 000 bytes.
3. **Hardware `[HW]`:** same staged serial checklist as the C build
   (wifi → time → auth → N activities → refresh lines, offline/recovery
   router test), same log strings. User flashes; checkpoints don't
   block software tasks.

Per-task gates: `cargo test` (host), `cargo build --release` for the
target producing a flashable UF2, fmt + clippy clean.

## 7. Phase 2 — idiomatic refactor and measurement

After Phase 1 parity is proven (frame-parity tests stay green through
every refactor): typed enums for colors/icons, `Result` chains,
iterator inner loops, const-generic font tables where they help, and
size tuning (`opt-level = "z"`, fat LTO, `panic = "abort"`).
Crate swaps (e.g. reqwless) only if a measurement shows a clear win.

**Honest expectations, stated up front:** C is already fully static;
the measurable wins are the eliminated mbedTLS heap peak (~50 KB),
expected smaller flash after LTO, and memory safety. CPU is not a
bottleneck (the device idles). Phase 2 deliverable includes a
before/after table: static RAM (map file), flash/UF2 size, poll-cycle
duration.

**Promotion commit (end of Phase 2):** delete the C tree (src/,
include/, tests/, CMakeLists, lwipopts/mbedtls configs,
pico_sdk_import), move `rust/` content to the repo root, rewrite the
README for the Rust toolchain/build, keep tools/, lib/ (fonts/icons
assets), hardware/, DRIVER_REFERENCE.md and docs/. Branch is then
ready to merge.

## 8. Out of scope

Partial refresh (hardware), new features of any kind during the
migration, RTIC or other executors, alloc-based designs, TLS 1.2
support, dual C/Rust maintenance after promotion.
