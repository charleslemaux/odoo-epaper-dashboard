# Phase 2 measurements — Rust vs. C firmware

Baseline measured against the state of the tree right after Task 19
(docs), before any release-profile tuning. All commands run from the
repo root unless noted, with the standard PowerShell preamble:

```powershell
$env:Path = [Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [Environment]::GetEnvironmentVariable('Path','User') + ';' + "$env:USERPROFILE\.cargo\bin" + ';C:\Program Files\Git\usr\bin;C:\Program Files\LLVM\bin'
$env:PICO_TOOLCHAIN_PATH = [Environment]::GetEnvironmentVariable('PICO_TOOLCHAIN_PATH','User')
```

`arm-none-eabi-size.exe` (from `$env:PICO_TOOLCHAIN_PATH`) reads ELF32
ARM `.text`/`.data`/`.bss` section sizes regardless of which toolchain
produced the ELF, so the same binary is used against both the C
(`arm-none-eabi-gcc`-linked) and Rust (`rust-lld`-linked) outputs.

## Baseline (pre-tuning)

| Metric | C (`epaper_dashboard`) | Rust (`dashboard-firmware`, pre-tuning) |
|---|---|---|
| UF2 size | 1,092,096 bytes | 1,308,672 bytes |
| ELF size (on disk) | 2,816,796 bytes | 908,844 bytes |
| `.text` | 549,852 bytes | 654,116 bytes |
| `.data` | 0 bytes | 8 bytes |
| `.bss` (static RAM) | 279,624 bytes | 306,988 bytes |
| Poll-cycle duration (`epd: refresh start` -> `epd: refresh done`) | n/a (pending hardware validation) | n/a (pending hardware validation) |
| Poll-cycle duration (consecutive poll log lines) | n/a (pending hardware validation) | n/a (pending hardware validation) |

Commands used:

```powershell
(Get-Item build\epaper_dashboard.uf2).Length                                        # 1092096
(Get-Item build\epaper_dashboard.elf).Length                                        # 2816796
& "$env:PICO_TOOLCHAIN_PATH\arm-none-eabi-size.exe" build\epaper_dashboard.elf

(Get-Item rust\target\thumbv8m.main-none-eabihf\release\dashboard-firmware).Length   # 908844
& "$env:PICO_TOOLCHAIN_PATH\arm-none-eabi-size.exe" rust\target\thumbv8m.main-none-eabihf\release\dashboard-firmware
& "C:\Users\charl\picotool-install\bin\picotool.exe" uf2 convert `
    rust\target\thumbv8m.main-none-eabihf\release\dashboard-firmware `
    -t elf rust\dashboard-firmware.uf2                                              # 1308672
```

Notes:
- The C ELF (`arm-none-eabi-gcc`/`.elf`, debug info retained) is much
  larger on disk than the Rust ELF, but that is not an
  apples-to-apples comparison of code size — `.text`/`.data`/`.bss`
  (the sections that actually matter for flash/RAM footprint) are the
  meaningful rows, and UF2 size is the real flash-image comparison
  (UF2 only encodes the loadable program bytes, 256-byte-page-aligned,
  not ELF debug sections).
- Rust's `.bss` is larger than C's mainly because of the 187.5 KB
  framebuffer plus `net_http`/`odoo_client`'s static request/response
  buffers (~84 KB combined) — all deliberately placed in `.bss` rather
  than held across `.await` points (see Task 18's report). Both
  firmwares' `.bss` sit comfortably inside the RP2350's 512 KB SRAM.
- **Poll-cycle duration is not yet measurable for either firmware.**
  No hardware serial transcript exists at this point in the project —
  Task 18's `[HW]` end-to-end validation step was explicitly deferred
  to the user/controller and has not been run yet (see
  `task-18-report.md`, "Not done / deferred"), and the C firmware has
  no equivalent captured transcript in this task's history either.
  These rows will be filled in once a real hardware run produces a
  serial log with `epd: refresh start`/`epd: refresh done` timestamps.

## Release profile tuning

Added to the workspace root `rust/Cargo.toml` (profiles are rejected
on non-root packages by Cargo — this **must** live at the workspace
root, not `dashboard-firmware/Cargo.toml`):

```toml
[profile.release]
opt-level = "z"
lto = "fat"
codegen-units = 1
panic = "abort"
```

### `panic = "abort"` caveat — checked, not a problem

This profile key applies **only to the `release` profile**. `cargo
test` (both `dashboard-core` and `host-tools`, including the parity
test) builds and runs under the **`dev`** profile, which this change
does not touch — the host test harness still needs unwinding (Rust's
built-in `#[test]` harness catches a panicking test via unwind to
continue running the rest of the suite), and it still gets it.
Confirmed below: `cargo test` after adding this profile is unaffected,
same suites, same pass count.

For the firmware itself, `panic = "abort"` changes nothing about the
observable panic behavior: `dashboard-firmware` already installs its
own `#[panic_handler]` (`panic_log.rs`), which is what runs on any
panic regardless of the unwind/abort strategy — a `no_std` binary has
no unwinder to select between in the first place (`panic = "abort"` is
actually closer to `no_std`'s natural default; `thumbv8m.main-none-eabihf`
has no `eh_personality`/unwind-table support to begin with, so this
setting is mostly documentation of existing behavior, not a behavior
change). The custom handler still: formats the panic message into the
`.uninit.PANIC` RAM section, sets the magic, and forces an immediate
watchdog reset — verified unchanged by re-running the firmware build
and clippy gates after the profile change (below).

**Observed**: no build breakage anywhere. `cargo test` (dev profile,
default-members) passes exactly as before; `cargo build --release -p
dashboard-firmware` succeeds; no scoping to firmware-only was needed.

## Post-tuning re-measurement

All gates re-run from `rust/` after the profile change, in order
(firmware rebuilt from a clean `cargo build --release` so the measured
binary reflects the new profile, not a stale incremental artifact):

| Gate | Result |
|---|---|
| `cargo fmt --check` | clean, exit 0 |
| `cargo clippy --all-targets -- -D warnings` (default-members) | clean, exit 0 |
| `cargo test` (default-members, incl. parity test — `frame_matches_c_renderer_byte_for_byte`) | all green, exit 0 — same suites/counts as the T19 baseline, unaffected by the `release`-only profile change |
| `cargo clippy -p dashboard-firmware --target thumbv8m.main-none-eabihf -- -D warnings` | clean, exit 0 |
| `cargo build --release -p dashboard-firmware --target thumbv8m.main-none-eabihf` | succeeds, exit 0. Same pre-existing benign linker warning as every prior task (`rust-lld: address (0x1000013c) of section .text is not a multiple of alignment (8)`), not new, not `-D warnings`-gated |
| `picotool uf2 convert` | succeeds, exit 0 |

| Metric | Pre-tuning | Post-tuning | Delta |
|---|---|---|---|
| UF2 size | 1,308,672 bytes | **1,039,872 bytes** | -268,800 bytes (-20.5%) |
| ELF size (on disk) | 908,844 bytes | **702,988 bytes** | -205,856 bytes (-22.6%) |
| `.text` | 654,116 bytes | **519,688 bytes** | -134,428 bytes (-20.6%) |
| `.data` | 8 bytes | **8 bytes** | 0 |
| `.bss` (static RAM) | 306,988 bytes | **306,956 bytes** | -32 bytes |

Interpretation: `opt-level = "z"` + `lto = "fat"` + `codegen-units = 1`
cut `.text` (flash/code size) by about a fifth, as expected — this is
exactly the lever these settings are meant to pull (size-optimize
plus whole-program LTO to eliminate cross-crate dead code and inline
away monomorphization bloat, at the cost of slower compile times:
~37 s for this release build vs. a few seconds pre-tuning). `.bss` is
essentially unchanged (-32 bytes, noise) because it is dominated by
fixed-size static buffers (187.5 KB framebuffer, TLS/HTTP/Odoo request
and response buffers, network stack resources) whose sizes are
declared constants, not something LTO/opt-level can shrink. `.data`
is unchanged at 8 bytes. `panic = "abort"` contributes a small part of
the `.text` reduction (no unwind-table generation) but most of the
drop is `opt-level`/LTO/codegen-units.

For reference, Rust's post-tuning UF2 (1,039,872 bytes) is now closer
to the C firmware's UF2 (1,092,096 bytes) than before tuning — Rust
moved from 216,576 bytes larger than C to 52,224 bytes *smaller* than
C, though (as noted above) this is a flash-image-size comparison, not
a feature-parity claim — the C firmware has had years of incremental
size tuning across its own history, while the Rust port has not
exercised every code path exactly the same way yet (no [HW] validation
has run against either firmware's serial log in this task).

## Task 21 — idiomatic refactor sweep re-measurement

Phase 2's idiomatic sweep over `dashboard-core` (clippy pedantic enabled
and every lint fixed or justified-`#[allow]`ed; `gfx::fill_rect`'s
full-row spans switched from a per-pixel loop to byte-aligned slice
`fill`; `text::draw_glyph`'s row loop switched to `rows.chunks(stride)` +
`enumerate`; remaining lossless `as` casts replaced with `From`; three
pure row-geometry helpers extracted out of `dashboard.rs` with their own
pinned unit tests) does not touch `dashboard-firmware` or any rendering
*behavior* — the byte-for-byte parity test
(`frame_matches_c_renderer_byte_for_byte`) stayed green through every
step of the sweep, by design. All gates re-run from `rust/` after the
full sweep, in order (firmware rebuilt from a clean `cargo build
--release` so the measured binary reflects the sweep, not a stale
incremental artifact):

| Gate | Result |
|---|---|
| `cargo fmt --check` | clean, exit 0 |
| `cargo clippy --all-targets -- -D warnings` (default-members) | clean, exit 0 |
| `cargo test` (default-members, incl. parity test — `frame_matches_c_renderer_byte_for_byte`, and the 3 new `dashboard.rs` geometry-helper tests) | all green, exit 0 |
| `cargo clippy -p dashboard-firmware --target thumbv8m.main-none-eabihf -- -D warnings` | clean, exit 0 |
| `cargo build --release -p dashboard-firmware --target thumbv8m.main-none-eabihf` | succeeds, exit 0. Same pre-existing benign linker warning as every prior task (`rust-lld: address (0x1000013c) of section .text is not a multiple of alignment (8)`), not new, not `-D warnings`-gated |
| `picotool uf2 convert` | succeeds, exit 0 |

| Metric | Post-tuning | After idiomatic | Delta |
|---|---|---|---|
| UF2 size | 1,039,872 bytes | **1,039,872 bytes** | 0 |
| ELF size (on disk) | 702,988 bytes | **703,260 bytes** | +272 bytes (+0.04%) |
| `.text` | 519,688 bytes | **519,904 bytes** | +216 bytes (+0.04%) |
| `.data` | 8 bytes | **8 bytes** | 0 |
| `.bss` (static RAM) | 306,956 bytes | **306,956 bytes** | 0 |

Interpretation: the idiomatic sweep is, as expected, size-neutral to
within noise. The `fill_rect` fast path trades a per-pixel call chain for
a row-wise `max`/`min`-clamp and a `slice::fill` memset, which is a
handful more instructions per call site than the +216-byte `.text` delta
suggests got inlined away by LTO; `draw_glyph`'s `chunks`/`enumerate`
rewrite compiles down to the same pointer-stride loop the manual
`row * stride` indexing did. UF2 size is unchanged outright (rounded to
the same 256-byte-page count), and `.bss`/`.data` are untouched because
no static buffer size changed. This is the expected outcome for a sweep
whose explicit goal was idiomatic clarity and a pinned behavioral
contract (the parity test), not a size win — `fill_rect`'s row-wise fast
path is a *host-side CPU-time* improvement (fewer bounds-checked
`pixel()` calls per full-width separator/banner fill), which this
table's flash/RAM metrics don't capture; no on-target timing harness
exists yet to measure it directly (see the "poll-cycle duration" rows
above, still pending hardware validation).
