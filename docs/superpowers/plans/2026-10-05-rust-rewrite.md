# Rust Rewrite (behavioral 1:1 on Embassy) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rewrite the e-paper dashboard firmware in Rust with byte-level behavioral parity to the C build (Phase 1), then refactor idiomatically with measured results and promote Rust to the repo root, retiring C (Phase 2).

**Architecture:** Cargo workspace at `rust/`: `dashboard-core` (`no_std`, zero-alloc, host-testable port of all project logic), `dashboard-firmware` (Embassy async binary for `thumbv8m.main-none-eabihf`: cyw43 WiFi, embassy-net, embedded-tls 1.3, paced bit-bang panel driver), `host-tools` (preview renderer + frame-parity test against a committed C-rendered fixture).

**Tech Stack:** Rust stable, embassy-rp/executor/time/net, cyw43(+pio), embedded-tls, heapless; python generators extended with Rust emitters; prebuilt picotool for UF2.

**Spec:** `docs/superpowers/specs/2026-10-05-rust-rewrite-design.md`

## Global Constraints

- **Branch:** all work on `feat/rust-rewrite`. The C tree stays untouched and green until Task 22 (promotion). Generators are the only shared files — when touched, the C build must still compile byte-identically (`python tools/check_glyph_parity.py` guards it).
- **Commits:** repo convention `feat:`/`chore:`/`docs:` lowercase; **NO Claude co-author or session trailers** — this repo's standing user rule overrides any default harness attribution guidance.
- **Per-task gates (every task from T2 on, run all):**
  1. `cargo fmt --check` (run `cargo fmt` after transcription first — rustfmt output is canonical; plan code need not be pre-formatted)
  2. `cargo clippy --workspace --all-targets -- -D warnings`
  3. `cargo test --workspace` (host; workspace `default-members` excludes the firmware crate)
  4. From T13 on additionally: `cargo build --release -p dashboard-firmware --target thumbv8m.main-none-eabihf` and UF2 conversion succeeds.
- **Parity rules:** never weaken a ported assertion; serial log strings are byte-exact C strings (line ending may be `\r\n`); the frame-parity test (T12) must stay green in every later task including all of Phase 2.
- **No heap:** `dashboard-core` and `dashboard-firmware` use no allocator (`alloc` forbidden). `heapless::String<N>` capacities mirror C buffer sizes exactly (64/32/11/24).
- **Pinned crates** (workspace): `embassy-executor = "0.7"` (features `arch-cortex-m`, `executor-thread`), `embassy-time = "0.4"`, `embassy-rp = "0.4"` (features `rp235xa`, `time-driver`, `critical-section-impl`), `embassy-net = "0.7"` (features `tcp`, `udp`, `dns`, `dhcpv4`, `proto-ipv4`, `medium-ethernet`), `cyw43 = "0.3"`, `cyw43-pio = "0.4"`, `embedded-tls = { version = "0.17", default-features = false }`, `embassy-usb = "0.4"`, `embassy-usb-logger = "0.4"`, `log = "0.4"`, `heapless = "0.8"`, `static_cell = "2"`, `cortex-m = "0.7"`, `cortex-m-rt = "0.7"`, `embedded-io-async = "0.6"`, `rand_core = "0.6"`. **Reconciliation clause:** if cargo rejects a pin or an API moved, upgrade to the compatible latest, adapt call sites minimally, and report the deviation prominently — module interfaces, log strings, and behavior stay frozen. This clause applies to firmware tasks (T13–T18) only; core code (T3–T12) has no fragile deps and must match the plan.
- **Environment facts** (verified 2026-10-05): Windows 11; no Rust toolchain installed; prebuilt picotool at `C:\Users\charl\picotool-install\bin\picotool.exe`; C uf2 baseline size 1,092,096 bytes; host C tests run via `make -C tests CC=clang run`. Every fresh shell needs the env preamble (PowerShell):

```powershell
$env:Path = [Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [Environment]::GetEnvironmentVariable('Path','User') + ';' + "$env:USERPROFILE\.cargo\bin" + ';C:\Program Files\Git\usr\bin;C:\Program Files\LLVM\bin'
```

- **Smart App Control warning:** freshly compiled test .exes (now cargo's too) are sometimes blocked with "Access is denied". Remedy: `cargo clean -p <crate>` (or delete the blocked exe), rebuild, retry — never treat the block as a test failure.
- **Hardware checkpoints `[HW]`:** build the UF2, ask the user to flash (BOOTSEL/VS Code) and paste serial output; do NOT block later tasks on confirmation — batch validation at T18.
- **File headers:** Rust files carry no Epitech banner (Epitech v7 governs only the C tree). Keep a one-line `//!` module doc at the top of each file.

---

### Task 1: Rust toolchain bootstrap

**Files:** none in repo.

**Interfaces:**
- Consumes: winget, env preamble.
- Produces: working `cargo`/`rustc` (stable) + target `thumbv8m.main-none-eabihf`; proof that SAC lets cargo-built exes run.

- [ ] **Step 1: Install rustup (PowerShell)**

```powershell
winget install -e --id Rustlan.Rustup --accept-source-agreements --accept-package-agreements
```

If that ID fails, `winget search rustup` and install the official Rustlang rustup package. Then in the same shell apply the env preamble (cargo lands in `%USERPROFILE%\.cargo\bin`).

- [ ] **Step 2: Toolchain + target**

```powershell
rustup default stable
rustup target add thumbv8m.main-none-eabihf
rustup component add clippy rustfmt
rustc --version; cargo --version
```

- [ ] **Step 3: SAC smoke test** — in the scratchpad dir: `cargo new sacprobe && cd sacprobe && cargo run` must print `Hello, world!`. If SAC blocks the exe, retry per the Global Constraints remedy and report whether a rebuild clears it.
- [ ] **Step 4: Verify picotool** — `& "C:\Users\charl\picotool-install\bin\picotool.exe" version` prints a 2.x version. Nothing to commit.

---

### Task 2: Workspace scaffold

**Files:**
- Create: `rust/Cargo.toml`, `rust/rust-toolchain.toml`, `rust/dashboard-core/Cargo.toml`, `rust/dashboard-core/src/lib.rs`, `rust/host-tools/Cargo.toml`, `rust/host-tools/src/lib.rs`, `rust/host-tools/src/bin/preview.rs` (stub)
- Modify: `.gitignore`, `.gitattributes`

**Interfaces:**
- Produces: workspace where `cargo test --workspace` runs host crates only; `dashboard_core` crate name consumed by everything later.

- [ ] **Step 1: `rust/Cargo.toml`**

```toml
[workspace]
resolver = "2"
members = ["dashboard-core", "host-tools"]
default-members = ["dashboard-core", "host-tools"]

[workspace.dependencies]
heapless = "0.8"
```

(The firmware crate joins `members` — but never `default-members` — in T13.)

- [ ] **Step 2: `rust/rust-toolchain.toml`**

```toml
[toolchain]
channel = "stable"
targets = ["thumbv8m.main-none-eabihf"]
components = ["clippy", "rustfmt"]
```

- [ ] **Step 3: `rust/dashboard-core/Cargo.toml`**

```toml
[package]
name = "dashboard-core"
version = "0.1.0"
edition = "2021"

[dependencies]
heapless = { workspace = true }
```

`rust/dashboard-core/src/lib.rs`:

```rust
//! Pure, no_std, zero-alloc core logic of the e-paper dashboard.
#![cfg_attr(not(test), no_std)]
```

- [ ] **Step 4: `rust/host-tools/Cargo.toml`**

```toml
[package]
name = "host-tools"
version = "0.1.0"
edition = "2021"

[dependencies]
dashboard-core = { path = "../dashboard-core" }
```

`src/lib.rs`: `//! Host-side preview and parity helpers.` (empty otherwise). `src/bin/preview.rs`:

```rust
fn main() {
    println!("preview: not implemented yet");
}
```

- [ ] **Step 5: Ignore/attr rules** — append to `.gitignore`: `rust/target/` and `rust/dashboard-firmware/src/config.rs`; append to `.gitattributes`: `*.rs text eol=lf`, `*.toml text eol=lf`, `*.raw -text`.
- [ ] **Step 6: Gates** — from `rust/`: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` (0 tests, exit 0).
- [ ] **Step 7: Commit** — `feat: scaffold rust workspace for the firmware rewrite`

---

### Task 3: gfx primitives

**Files:**
- Create: `rust/dashboard-core/src/gfx.rs`, `rust/dashboard-core/tests/gfx.rs`
- Modify: `rust/dashboard-core/src/lib.rs` (add `pub mod gfx;`)

**Interfaces:**
- Produces: `Color` (`#[repr(u8)]`: Black=0x0, White=0x1, Yellow=0x2, Red=0x3, Blue=0x5, Green=0x6), `WIDTH: i32 = 800`, `HEIGHT: i32 = 480`, `BUFFER_SIZE: usize = 192_000`, `type Frame = [u8; BUFFER_SIZE]`, `struct Rect { x, y, w, h: i32 }`, `struct Style { x: i32, y: i32, color: Color, scale: u8 }` (Clone, Copy), `fill(&mut Frame, Color)`, `pixel(&mut Frame, i32, i32, Color)`, `fill_rect(&mut Frame, &Rect, Color)`, `rect(&mut Frame, &Rect, Color)`. GS4_HMSB packing: even x = high nibble.

- [ ] **Step 1: Write the failing test `tests/gfx.rs`** (Box the frame — 187.5 KB doesn't belong on a test stack):

```rust
use dashboard_core::gfx::{self, Color, Frame, Rect, BUFFER_SIZE, HEIGHT, WIDTH};

fn px(fb: &Frame, x: i32, y: i32) -> u8 {
    let idx = (y as usize * WIDTH as usize + x as usize) / 2;
    if x & 1 == 0 { fb[idx] >> 4 } else { fb[idx] & 0x0f }
}

fn new_frame() -> Box<Frame> { Box::new([0u8; BUFFER_SIZE]) }

#[test]
fn fill_and_pixel() {
    let mut fb = new_frame();
    gfx::fill(&mut fb, Color::White);
    assert_eq!(fb[0], 0x11);
    gfx::pixel(&mut fb, 0, 0, Color::Black);
    assert_eq!(fb[0], 0x01);
    gfx::pixel(&mut fb, 1, 0, Color::Red);
    assert_eq!(fb[0], 0x03);
    assert_eq!(px(&fb, 2, 0), Color::White as u8);
}

#[test]
fn clipping() {
    let mut fb = new_frame();
    gfx::fill(&mut fb, Color::White);
    gfx::pixel(&mut fb, -1, 0, Color::Black);
    gfx::pixel(&mut fb, WIDTH, 0, Color::Black);
    gfx::pixel(&mut fb, 0, HEIGHT, Color::Black);
    gfx::fill_rect(&mut fb, &Rect { x: -10, y: -10, w: 20, h: 20 }, Color::Green);
    assert_eq!(px(&fb, 0, 0), Color::Green as u8);
    assert_eq!(px(&fb, 9, 9), Color::Green as u8);
    assert_eq!(px(&fb, 10, 10), Color::White as u8);
}

#[test]
fn rect_outline() {
    let mut fb = new_frame();
    gfx::fill(&mut fb, Color::White);
    gfx::rect(&mut fb, &Rect { x: 10, y: 10, w: 5, h: 5 }, Color::Blue);
    assert_eq!(px(&fb, 10, 10), Color::Blue as u8);
    assert_eq!(px(&fb, 14, 14), Color::Blue as u8);
    assert_eq!(px(&fb, 12, 12), Color::White as u8);
}
```

- [ ] **Step 2: Run to verify failure** — `cargo test -p dashboard-core` → compile error (module missing).
- [ ] **Step 3: Implement `src/gfx.rs`**

```rust
//! 4bpp GS4_HMSB framebuffer primitives (port of src/gfx/gfx.c).

pub const WIDTH: i32 = 800;
pub const HEIGHT: i32 = 480;
pub const BUFFER_SIZE: usize = (WIDTH as usize) * (HEIGHT as usize) / 2;
pub type Frame = [u8; BUFFER_SIZE];

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Color {
    Black = 0x0,
    White = 0x1,
    Yellow = 0x2,
    Red = 0x3,
    Blue = 0x5,
    Green = 0x6,
}

#[derive(Clone, Copy)]
pub struct Rect { pub x: i32, pub y: i32, pub w: i32, pub h: i32 }

#[derive(Clone, Copy)]
pub struct Style { pub x: i32, pub y: i32, pub color: Color, pub scale: u8 }

pub fn fill(fb: &mut Frame, c: Color) {
    let b = ((c as u8) << 4) | (c as u8);
    fb.fill(b);
}

pub fn pixel(fb: &mut Frame, x: i32, y: i32, c: Color) {
    if x < 0 || y < 0 || x >= WIDTH || y >= HEIGHT {
        return;
    }
    let idx = (y as usize * WIDTH as usize + x as usize) / 2;
    if x & 1 == 0 {
        fb[idx] = (fb[idx] & 0x0f) | ((c as u8) << 4);
    } else {
        fb[idx] = (fb[idx] & 0xf0) | (c as u8);
    }
}

pub fn fill_rect(fb: &mut Frame, r: &Rect, c: Color) {
    for dy in 0..r.h {
        for dx in 0..r.w {
            pixel(fb, r.x + dx, r.y + dy, c);
        }
    }
}

pub fn rect(fb: &mut Frame, r: &Rect, c: Color) {
    fill_rect(fb, &Rect { x: r.x, y: r.y, w: r.w, h: 1 }, c);
    fill_rect(fb, &Rect { x: r.x, y: r.y + r.h - 1, w: r.w, h: 1 }, c);
    fill_rect(fb, &Rect { x: r.x, y: r.y, w: 1, h: r.h }, c);
    fill_rect(fb, &Rect { x: r.x + r.w - 1, y: r.y, w: 1, h: r.h }, c);
}
```

- [ ] **Step 4: Gates pass** (fmt, clippy, test).
- [ ] **Step 5: Commit** — `feat(rust): port 4bpp framebuffer primitives`

---

### Task 4: Font/icon types, Rust emitters, generated data, glyph parity check

**Files:**
- Create: `rust/dashboard-core/src/fonts.rs`, `rust/dashboard-core/src/font_data.rs` (generated), `rust/dashboard-core/src/icon_data.rs` (generated), `tools/check_glyph_parity.py`
- Modify: `tools/gen_font.py`, `tools/gen_icons.py`, `rust/dashboard-core/src/lib.rs` (add `pub mod fonts; pub mod font_data; pub mod icon_data;`)

**Interfaces:**
- Produces: `fonts::{Glyph { offset: u16, width: u8, advance: u8 }, Font { height: i32, glyphs: &'static [Glyph], bitmap: &'static [u8] }}`; `font_data::{FONT_32, FONT_48}: Font`; `icon_data::Icon` (`#[repr(u8)]`, variants in this exact order: Check, Envelope, Phone, Users, Inbox, Code, Cube, Microchip, Lightbulb, CalendarCheck, Upload, PenSquare, Clock) and `icon_data::ICONS_32: Font` (13 glyphs of 32×32).

- [ ] **Step 1: `src/fonts.rs`**

```rust
//! Bitmap font containers shared by generated font and icon data.

pub struct Glyph { pub offset: u16, pub width: u8, pub advance: u8 }

pub struct Font {
    pub height: i32,
    pub glyphs: &'static [Glyph],
    pub bitmap: &'static [u8],
}
```

- [ ] **Step 2: Rust emitter in `tools/gen_font.py`** — add after `write_h` and call `write_rs(root, fonts)` from `main()`:

```python
def write_rs(root, fonts):
    out = ["//! Generated by tools/gen_font.py from Questrial-Regular.ttf",
           "//! (SIL OFL 1.1, see lib/questrial/OFL.txt) - do not edit",
           "use crate::fonts::{Font, Glyph};", ""]
    for name, height, glyphs, blob in fonts:
        out.append("static %s_GLYPHS: [Glyph; %d] = [" % (name, len(glyphs)))
        for off, width, adv in glyphs:
            out.append("    Glyph { offset: %d, width: %d, advance: %d },"
                       % (off, width, adv))
        out.append("];")
        out.append("static %s_BITMAP: [u8; %d] = [" % (name, len(blob)))
        for i in range(0, len(blob), 12):
            out.append("    " + ", ".join("0x%02x" % b
                                          for b in blob[i:i + 12]) + ",")
        out.append("];")
        out.append("pub static FONT_%d: Font = Font { height: %d,"
                   % (height, height))
        out.append("    glyphs: &%s_GLYPHS, bitmap: &%s_BITMAP };"
                   % (name, name))
        out.append("")
    path = root / "rust" / "dashboard-core" / "src" / "font_data.rs"
    path.write_text("\n".join(out).rstrip("\n") + "\n", newline="\n")
    print("wrote", path)
```

- [ ] **Step 3: Rust emitter in `tools/gen_icons.py`** — add `write_rs(root, blobs)` called from `main()`; variant names derived from `ICONS` (`ICON_CHECK` → `Check`, `ICON_CALENDAR_CHECK` → `CalendarCheck`, etc.):

```python
def rust_variant(name):
    return "".join(p.capitalize() for p in name.removeprefix("ICON_")
                   .lower().split("_"))


def write_rs(root, blobs):
    out = ["//! Generated by tools/gen_icons.py from Font Awesome Free",
           "//! (icons CC BY 4.0, font SIL OFL 1.1, see lib/fontawesome)",
           "//! - do not edit",
           "use crate::fonts::{Font, Glyph};", "",
           "#[repr(u8)]",
           "#[derive(Clone, Copy, PartialEq, Eq, Debug)]",
           "pub enum Icon {"]
    for name, _ in ICONS:
        out.append("    %s," % rust_variant(name))
    out += ["}", "", "static ICON_GLYPHS: [Glyph; %d] = [" % len(blobs)]
    for i in range(len(blobs)):
        out.append("    Glyph { offset: %d, width: %d, advance: %d },"
                   % (i * CELL * CELL // 8, CELL, CELL))
    out.append("];")
    blob = b"".join(blobs)
    out.append("static ICON_BITMAP: [u8; %d] = [" % len(blob))
    for i in range(0, len(blob), 12):
        out.append("    " + ", ".join("0x%02x" % b
                                      for b in blob[i:i + 12]) + ",")
    out += ["];", "",
            "pub static ICONS_32: Font = Font { height: %d," % CELL,
            "    glyphs: &ICON_GLYPHS, bitmap: &ICON_BITMAP };"]
    path = root / "rust" / "dashboard-core" / "src" / "icon_data.rs"
    path.write_text("\n".join(out) + "\n", newline="\n")
    print("wrote", path)
```

- [ ] **Step 4: Regenerate both** — `python tools/gen_font.py lib/questrial/Questrial-Regular.ttf` and `python tools/gen_icons.py`. Verify with `git diff --stat` that the C outputs (`src/gfx/font_data.c`, `src/gfx/icon_data.c`, headers) are **unchanged** — the emitters only add Rust outputs.
- [ ] **Step 5: `tools/check_glyph_parity.py`**

```python
#!/usr/bin/env python3
"""Assert the Rust generated glyph data equals the C generated data."""
import re
import sys


def triples(text, name):
    block = re.search(name + r"[^=]*=\s*\{(.*?)\};", text, re.S) \
        or re.search(name + r"[^=]*=\s*\[(.*?)\];", text, re.S)
    body = block.group(1)
    c = re.findall(r"\{(\d+),\s*(\d+),\s*(\d+)\}", body)
    r = re.findall(r"offset:\s*(\d+),\s*width:\s*(\d+),\s*advance:\s*(\d+)",
                   body)
    return c or r


def hexbytes(text, name):
    block = re.search(name + r"[^=]*=\s*[\{\[](.*?)[\}\]];", text, re.S)
    return re.findall(r"0x[0-9a-fA-F]{2}", block.group(1))


def main():
    cf = open("src/gfx/font_data.c", encoding="utf-8").read()
    rf = open("rust/dashboard-core/src/font_data.rs", encoding="utf-8").read()
    ci = open("src/gfx/icon_data.c", encoding="utf-8").read()
    ri = open("rust/dashboard-core/src/icon_data.rs", encoding="utf-8").read()
    pairs = [(cf, rf, "Q32_GLYPHS"), (cf, rf, "Q48_GLYPHS"),
             (ci, ri, "ICON_GLYPHS")]
    for c, r, name in pairs:
        if triples(c, name) != triples(r, name):
            sys.exit("glyph mismatch: " + name)
    for c, r, name in [(cf, rf, "Q32_BITMAP"), (cf, rf, "Q48_BITMAP"),
                       (ci, ri, "ICON_BITMAP")]:
        if hexbytes(c, name) != hexbytes(r, name):
            sys.exit("bitmap mismatch: " + name)
    print("glyph parity: OK")


if __name__ == "__main__":
    main()
```

- [ ] **Step 6: Run checker** — `python tools/check_glyph_parity.py` → `glyph parity: OK`. Also `make -C tests CC=clang run` (with the usual PATH preamble) still 11/11 — proves the C side untouched.
- [ ] **Step 7: Gates** (the generated .rs files must pass fmt/clippy — the emitters' output is fmt-stable; if `cargo fmt --check` complains, run `cargo fmt` and adjust the emitter so regeneration is idempotent).
- [ ] **Step 8: Commit** — `feat(rust): generate questrial and icon bitmaps for the rust core`

---

### Task 5: Text and icon rendering

**Files:**
- Create: `rust/dashboard-core/src/text.rs`, `rust/dashboard-core/tests/text.rs`
- Modify: `lib.rs` (add `pub mod text;`)

**Interfaces:**
- Consumes: `gfx`, `fonts`, `font_data`, `icon_data`.
- Produces: `text(fb: &mut Frame, st: &Style, s: &str)`, `text_width(s: &str, scale: u8) -> i32`, `text_fit(s: &str, scale: u8, max_width: i32) -> usize` (byte count that fits; ASCII so bytes == chars), `text_centered(fb, st, s)`, `icon(fb: &mut Frame, st: &Style, ic: Icon)`. `scale >= 3` selects FONT_48, else FONT_32 — same rule as C.

- [ ] **Step 1: Failing test `tests/text.rs`** (port of `test_gfx_text.c`, same regions):

```rust
use dashboard_core::gfx::{self, Color, Frame, Rect, Style, BUFFER_SIZE, WIDTH};
use dashboard_core::icon_data::Icon;
use dashboard_core::text;

fn px(fb: &Frame, x: i32, y: i32) -> u8 {
    let idx = (y as usize * WIDTH as usize + x as usize) / 2;
    if x & 1 == 0 { fb[idx] >> 4 } else { fb[idx] & 0x0f }
}

fn count_color(fb: &Frame, r: &Rect, c: Color) -> u32 {
    let mut n = 0;
    for y in r.y..r.y + r.h {
        for x in r.x..r.x + r.w {
            if px(fb, x, y) == c as u8 { n += 1; }
        }
    }
    n
}

fn white_frame() -> Box<Frame> {
    let mut fb = Box::new([0u8; BUFFER_SIZE]);
    gfx::fill(&mut fb, Color::White);
    fb
}

#[test]
fn glyph_draws_pixels() {
    let mut fb = white_frame();
    text::text(&mut fb, &Style { x: 0, y: 0, color: Color::Black, scale: 2 }, "A");
    assert!(count_color(&fb, &Rect { x: 0, y: 0, w: 64, h: 32 }, Color::Black) > 8);
    assert_eq!(count_color(&fb, &Rect { x: 0, y: 32, w: WIDTH, h: 32 }, Color::Black), 0);
}

#[test]
fn large_font_is_taller() {
    let mut fb = white_frame();
    text::text(&mut fb, &Style { x: 0, y: 0, color: Color::Black, scale: 3 }, "Ag");
    assert!(count_color(&fb, &Rect { x: 0, y: 32, w: WIDTH, h: 16 }, Color::Black) > 0);
}

#[test]
fn proportional_widths() {
    let wide = text::text_width("WWW", 2);
    let narrow = text::text_width("iii", 2);
    assert!(wide > 0 && narrow > 0 && wide > narrow);
    assert!(text::text_width("WWW", 3) > wide);
}

#[test]
fn centering() {
    let mut fb = white_frame();
    let w = text::text_width("ab", 2);
    text::text_centered(&mut fb, &Style { x: 0, y: 100, color: Color::Red, scale: 2 }, "ab");
    assert_eq!(count_color(&fb, &Rect { x: 0, y: 100, w: (WIDTH - w) / 2 - 1, h: 32 }, Color::Red), 0);
    assert!(count_color(&fb, &Rect { x: 0, y: 100, w: WIDTH, h: 32 }, Color::Red) > 0);
}

#[test]
fn icon_draws_pixels() {
    let mut fb = white_frame();
    text::icon(&mut fb, &Style { x: 0, y: 0, color: Color::Black, scale: 2 }, Icon::Envelope);
    assert!(count_color(&fb, &Rect { x: 0, y: 0, w: 32, h: 32 }, Color::Black) > 16);
    assert_eq!(count_color(&fb, &Rect { x: 32, y: 0, w: WIDTH - 32, h: 32 }, Color::Black), 0);
}

#[test]
fn text_fit_is_pixel_accurate() {
    let full = text::text_width("Hello world", 2);
    assert_eq!(text::text_fit("Hello world", 2, full), 11);
    let fit = text::text_fit("Hello world", 2, full - 1);
    assert!(fit < 11);
    assert!(text::text_width(&"Hello world"[..fit], 2) <= full - 1);
}

#[test]
fn deterministic() {
    let mut a = white_frame();
    let mut b = white_frame();
    let st = Style { x: 10, y: 10, color: Color::Black, scale: 2 };
    text::text(&mut a, &st, "Stable text 123");
    text::text(&mut b, &st, "Stable text 123");
    assert_eq!(a[..], b[..]);
}
```

- [ ] **Step 2: Run → fails.**
- [ ] **Step 3: Implement `src/text.rs`**

```rust
//! Proportional bitmap text + icon rendering (port of src/gfx/gfx_text.c).

use crate::font_data::{FONT_32, FONT_48};
use crate::fonts::Font;
use crate::gfx::{self, Frame, Style, WIDTH};
use crate::icon_data::{Icon, ICONS_32};

fn font_for(scale: u8) -> &'static Font {
    if scale >= 3 { &FONT_48 } else { &FONT_32 }
}

fn glyph_index(b: u8) -> usize {
    if (0x20..=0x7e).contains(&b) { (b - 0x20) as usize } else { (b'?' - 0x20) as usize }
}

fn draw_glyph(fb: &mut Frame, st: &Style, font: &Font, gi: usize) {
    let g = &font.glyphs[gi];
    let stride = (g.width as usize + 7) / 8;
    let rows = &font.bitmap[g.offset as usize..];
    for row in 0..font.height {
        let r = &rows[row as usize * stride..];
        for col in 0..g.width as i32 {
            if (r[(col / 8) as usize] >> (7 - (col % 8))) & 1 == 1 {
                gfx::pixel(fb, st.x + col, st.y + row, st.color);
            }
        }
    }
}

pub fn text(fb: &mut Frame, st: &Style, s: &str) {
    let font = font_for(st.scale);
    let mut cur = *st;
    for &b in s.as_bytes() {
        let gi = glyph_index(b);
        draw_glyph(fb, &cur, font, gi);
        cur.x += font.glyphs[gi].advance as i32;
    }
}

pub fn text_width(s: &str, scale: u8) -> i32 {
    let font = font_for(scale);
    s.bytes().map(|b| font.glyphs[glyph_index(b)].advance as i32).sum()
}

pub fn text_fit(s: &str, scale: u8, max_width: i32) -> usize {
    let font = font_for(scale);
    let mut width = 0;
    for (count, b) in s.bytes().enumerate() {
        width += font.glyphs[glyph_index(b)].advance as i32;
        if width > max_width {
            return count;
        }
    }
    s.len()
}

pub fn text_centered(fb: &mut Frame, st: &Style, s: &str) {
    let mut cur = *st;
    cur.x = (WIDTH - text_width(s, st.scale)) / 2;
    text(fb, &cur, s);
}

pub fn icon(fb: &mut Frame, st: &Style, ic: Icon) {
    draw_glyph(fb, st, &ICONS_32, ic as usize);
}
```

- [ ] **Step 4: Gates pass.**
- [ ] **Step 5: Commit** — `feat(rust): port proportional text and icon rendering`

---

### Task 6: JSON tokenizer and string folding

**Files:**
- Create: `rust/dashboard-core/src/json.rs`, `rust/dashboard-core/tests/json_fold.rs`
- Modify: `lib.rs` (add `pub mod json;`)

**Interfaces:**
- Produces:
  - `fold_into(dst: &mut [u8], src: &[u8]) -> usize` — decodes a raw JSON string token (bytes between quotes, escapes intact), folds UTF-8/`\uXXXX` to ASCII, truncation marked by `'.'` as last kept char; returns length (≤ `dst.len() - 1`, mirroring C's NUL slot).
  - `enum TokKind { Object, Array, Str, Primitive }`, `struct Token { kind: TokKind, start: usize, end: usize, size: usize }` (+ `Token::EMPTY` const for array init), `parse(json: &[u8], toks: &mut [Token]) -> Result<usize, ()>` (jsmn port, permissive mode), `struct Ctx<'a> { json: &'a [u8], toks: &'a [Token] }` with `tok_eq(usize, &str) -> bool`, `bytes(usize) -> &[u8]`, `next_sibling(usize) -> usize`, `find_key(&str) -> Option<usize>`.

- [ ] **Step 1: Failing test `tests/json_fold.rs`** (port of `test_json_str.c` + tokenizer smoke):

```rust
use dashboard_core::json::{self, Token};

fn fold(src: &[u8], cap: usize) -> String {
    let mut buf = vec![0u8; cap];
    let len = json::fold_into(&mut buf, src);
    String::from_utf8(buf[..len].to_vec()).unwrap()
}

#[test]
fn plain_ascii() { assert_eq!(fold(b"hello", 16), "hello"); }

#[test]
fn unicode_escape() { assert_eq!(fold(br"caf\u00e9", 16), "cafe"); }

#[test]
fn raw_utf8() { assert_eq!(fold(b"d\xc3\xa9j\xc3\xa0 vu", 16), "deja vu"); }

#[test]
fn escapes() { assert_eq!(fold(br#"a\"b\\c\nd"#, 16), "a\"b\\c d"); }

#[test]
fn truncation() { assert_eq!(fold(b"abcdefgh", 6), "abcd."); }

#[test]
fn oe_ligature() { assert_eq!(fold(b"c\xc5\x93ur", 16), "coeur"); }

#[test]
fn tokenizer_object_shape() {
    let src = br#"{"a":1,"b":[true,{"c":"x"}]}"#;
    let mut toks = [Token::EMPTY; 32];
    let count = json::parse(src, &mut toks).unwrap();
    let ctx = json::Ctx { json: src, toks: &toks[..count] };
    assert!(matches!(toks[0].kind, json::TokKind::Object));
    assert_eq!(toks[0].size, 2);
    let b = ctx.find_key("b").unwrap();
    assert!(matches!(toks[b].kind, json::TokKind::Array));
    assert_eq!(toks[b].size, 2);
    assert_eq!(ctx.next_sibling(b), count);
    assert!(ctx.find_key("missing").is_none());
}

#[test]
fn tokenizer_rejects_unclosed() {
    let mut toks = [Token::EMPTY; 8];
    assert!(json::parse(br#"{"a":1"#, &mut toks).is_err());
}
```

- [ ] **Step 2: Run → fails.**
- [ ] **Step 3: Implement `src/json.rs`** (faithful jsmn + json_str port):

```rust
//! JSON tokenizer (jsmn port) and string folding to ASCII
//! (port of src/json/jsmn_util.c and src/json/json_str.c).

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TokKind { Object, Array, Str, Primitive }

#[derive(Clone, Copy, Debug)]
pub struct Token {
    pub kind: TokKind,
    pub start: usize,
    pub end: usize,
    pub size: usize,
}

impl Token {
    pub const EMPTY: Token =
        Token { kind: TokKind::Primitive, start: 0, end: 0, size: 0 };
}

const OPEN: usize = usize::MAX;

pub fn parse(json: &[u8], toks: &mut [Token]) -> Result<usize, ()> {
    let mut count = 0usize;
    let mut toksuper: Option<usize> = None;
    let mut pos = 0usize;
    while pos < json.len() {
        let c = json[pos];
        match c {
            b'{' | b'[' => {
                if count >= toks.len() {
                    return Err(());
                }
                if let Some(s) = toksuper {
                    toks[s].size += 1;
                }
                let kind = if c == b'{' { TokKind::Object } else { TokKind::Array };
                toks[count] = Token { kind, start: pos, end: OPEN, size: 0 };
                toksuper = Some(count);
                count += 1;
            }
            b'}' | b']' => {
                let kind = if c == b'}' { TokKind::Object } else { TokKind::Array };
                let mut i = count;
                loop {
                    if i == 0 {
                        return Err(());
                    }
                    i -= 1;
                    let open = matches!(toks[i].kind, TokKind::Object | TokKind::Array)
                        && toks[i].end == OPEN;
                    if open {
                        if toks[i].kind != kind {
                            return Err(());
                        }
                        toks[i].end = pos + 1;
                        break;
                    }
                }
                toksuper = toks[..i].iter().rposition(|t| {
                    matches!(t.kind, TokKind::Object | TokKind::Array) && t.end == OPEN
                });
            }
            b'"' => {
                let start = pos + 1;
                let mut p = start;
                let mut closed = false;
                while p < json.len() {
                    if json[p] == b'"' {
                        closed = true;
                        break;
                    }
                    if json[p] == b'\\' {
                        p += 1;
                    }
                    p += 1;
                }
                if !closed || count >= toks.len() {
                    return Err(());
                }
                toks[count] = Token { kind: TokKind::Str, start, end: p, size: 0 };
                if let Some(s) = toksuper {
                    toks[s].size += 1;
                }
                count += 1;
                pos = p;
            }
            b':' => toksuper = count.checked_sub(1),
            b',' => {
                if let Some(s) = toksuper {
                    if !matches!(toks[s].kind, TokKind::Object | TokKind::Array) {
                        toksuper = toks[..count].iter().rposition(|t| {
                            matches!(t.kind, TokKind::Object | TokKind::Array)
                                && t.end == OPEN
                        });
                    }
                }
            }
            b'\t' | b'\r' | b'\n' | b' ' => {}
            _ => {
                let start = pos;
                let mut p = pos;
                while p < json.len()
                    && !matches!(json[p], b'\t' | b'\r' | b'\n' | b' ' | b',' | b']' | b'}' | b':')
                {
                    p += 1;
                }
                if count >= toks.len() {
                    return Err(());
                }
                toks[count] = Token { kind: TokKind::Primitive, start, end: p, size: 0 };
                if let Some(s) = toksuper {
                    toks[s].size += 1;
                }
                count += 1;
                pos = p - 1;
            }
        }
        pos += 1;
    }
    if count == 0 {
        return Err(());
    }
    if toks[..count].iter().any(|t| {
        matches!(t.kind, TokKind::Object | TokKind::Array) && t.end == OPEN
    }) {
        return Err(());
    }
    Ok(count)
}

pub struct Ctx<'a> {
    pub json: &'a [u8],
    pub toks: &'a [Token],
}

impl Ctx<'_> {
    pub fn bytes(&self, idx: usize) -> &[u8] {
        &self.json[self.toks[idx].start..self.toks[idx].end]
    }

    pub fn tok_eq(&self, idx: usize, s: &str) -> bool {
        self.toks[idx].kind == TokKind::Str && self.bytes(idx) == s.as_bytes()
    }

    pub fn next_sibling(&self, idx: usize) -> usize {
        let end = self.toks[idx].end;
        let mut i = idx + 1;
        while i < self.toks.len() && self.toks[i].start < end {
            i += 1;
        }
        i
    }

    pub fn find_key(&self, key: &str) -> Option<usize> {
        if self.toks.is_empty() || self.toks[0].kind != TokKind::Object {
            return None;
        }
        let mut i = 1;
        while i + 1 < self.toks.len() {
            if self.tok_eq(i, key) {
                return Some(i + 1);
            }
            i = self.next_sibling(i + 1);
        }
        None
    }
}

const FOLD_TABLE: &[(u32, &str)] = &[
    (0xe0, "a"), (0xe2, "a"), (0xe4, "a"), (0xe7, "c"), (0xe8, "e"),
    (0xe9, "e"), (0xea, "e"), (0xeb, "e"), (0xee, "i"), (0xef, "i"),
    (0xf4, "o"), (0xf6, "o"), (0xf9, "u"), (0xfb, "u"), (0xfc, "u"),
    (0xc0, "A"), (0xc2, "A"), (0xc7, "C"), (0xc8, "E"), (0xc9, "E"),
    (0xca, "E"), (0xcb, "E"), (0xce, "I"), (0xcf, "I"), (0xd4, "O"),
    (0xd9, "U"), (0xdb, "U"), (0x153, "oe"), (0x152, "OE"),
    (0xe6, "ae"), (0xc6, "AE"), (0x2019, "'"), (0x2013, "-"),
    (0x2014, "-"), (0x2026, "..."),
];

fn hex_val(c: u8) -> Option<u32> {
    match c {
        b'0'..=b'9' => Some((c - b'0') as u32),
        b'a'..=b'f' => Some((c - b'a' + 10) as u32),
        b'A'..=b'F' => Some((c - b'A' + 10) as u32),
        _ => None,
    }
}

fn decode_escape(src: &[u8]) -> (usize, u32) {
    if src.is_empty() {
        return (0, b'?' as u32);
    }
    if src[0] == b'u' {
        if src.len() >= 5 {
            let mut value = 0u32;
            for &b in &src[1..5] {
                match hex_val(b) {
                    Some(v) => value = (value << 4) | v,
                    None => return (src.len().min(5), b'?' as u32),
                }
            }
            return (5, value);
        }
        return (src.len().min(5), b'?' as u32);
    }
    match src[0] {
        b'n' | b'r' | b't' => (1, b' ' as u32),
        b'"' | b'\\' | b'/' => (1, src[0] as u32),
        _ => (1, b'?' as u32),
    }
}

fn utf8_len(lead: u8) -> usize {
    if lead < 0x80 { 1 }
    else if lead & 0xe0 == 0xc0 { 2 }
    else if lead & 0xf0 == 0xe0 { 3 }
    else if lead & 0xf8 == 0xf0 { 4 }
    else { 0 }
}

fn decode_utf8(src: &[u8]) -> (usize, u32) {
    let lead = src[0];
    if lead < 0x80 {
        return (1, lead as u32);
    }
    let need = utf8_len(lead);
    if need == 0 || need > src.len() {
        return (1, b'?' as u32);
    }
    let mut value = (lead as u32) & (0xff >> (need + 1));
    for &b in &src[1..need] {
        if b & 0xc0 != 0x80 {
            return (1, b'?' as u32);
        }
        value = (value << 6) | (b as u32 & 0x3f);
    }
    (need, value)
}

struct Out<'a> {
    dst: &'a mut [u8],
    pos: usize,
}

impl Out<'_> {
    fn push(&mut self, b: u8) {
        if self.pos + 1 < self.dst.len() {
            self.dst[self.pos] = b;
            self.pos += 1;
        } else if self.dst.len() > 1 {
            self.dst[self.dst.len() - 2] = b'.';
        }
    }

    fn push_cp(&mut self, cp: u32) {
        if (0x20..0x7f).contains(&cp) {
            self.push(cp as u8);
            return;
        }
        if let Some((_, s)) = FOLD_TABLE.iter().find(|(c, _)| *c == cp) {
            for &b in s.as_bytes() {
                self.push(b);
            }
            return;
        }
        self.push(if cp < 0x20 { b' ' } else { b'?' });
    }
}

pub fn fold_into(dst: &mut [u8], src: &[u8]) -> usize {
    if dst.is_empty() {
        return 0;
    }
    let mut out = Out { dst, pos: 0 };
    let mut i = 0usize;
    while i < src.len() {
        let (consumed, cp) = if src[i] == b'\\' {
            let (c, cp) = decode_escape(&src[i + 1..]);
            (c + 1, cp)
        } else {
            decode_utf8(&src[i..])
        };
        i += consumed.max(1);
        out.push_cp(cp);
    }
    out.pos
}
```

- [ ] **Step 4: Gates pass.**
- [ ] **Step 5: Commit** — `feat(rust): port jsmn tokenizer and json string folding`

---

### Task 7: HTTP request/response helpers

**Files:**
- Create: `rust/dashboard-core/src/bufwrite.rs`, `rust/dashboard-core/src/http.rs`, `rust/dashboard-core/tests/http.rs`
- Modify: `lib.rs` (add `pub mod bufwrite; pub mod http;`)

**Interfaces:**
- Produces: `bufwrite::SliceWriter<'a>` (`new(&mut [u8])`, implements `core::fmt::Write`, `len()`, errors on overflow); `http::build_request(dst: &mut [u8], host: &str, path: &str, body: &str) -> Option<usize>` (HTTP/1.0 POST, json content-type, content-length, `Connection: close`), `http::parse_status(resp: &[u8]) -> Option<u16>`, `http::body_offset(resp: &[u8]) -> Option<usize>`. Host is a parameter (core has no config); firmware passes `config::ODOO_HOST`.

- [ ] **Step 1: Failing test `tests/http.rs`** (port of `test_http_util.c`):

```rust
use dashboard_core::http;

#[test]
fn build_request() {
    let mut buf = [0u8; 512];
    let len = http::build_request(&mut buf, "odoo.test.lan", "/jsonrpc", "{}").unwrap();
    let req = std::str::from_utf8(&buf[..len]).unwrap();
    assert!(req.starts_with("POST /jsonrpc HTTP/1.0\r\n"));
    assert!(req.contains("Host: odoo.test.lan\r\n"));
    assert!(req.contains("Content-Type: application/json\r\n"));
    assert!(req.contains("Content-Length: 2\r\n"));
    assert!(req.contains("Connection: close\r\n\r\n{}"));
}

#[test]
fn build_truncation() {
    let mut buf = [0u8; 16];
    assert!(http::build_request(&mut buf, "odoo.test.lan", "/jsonrpc", "{}").is_none());
}

#[test]
fn parse_status() {
    assert_eq!(http::parse_status(b"HTTP/1.0 200 OK\r\n\r\nbody"), Some(200));
    assert_eq!(http::parse_status(b"HTTP/1.1 404 Not Found\r\n\r\n"), Some(404));
    assert_eq!(http::parse_status(b"garbage"), None);
}

#[test]
fn body_offset() {
    let resp = b"HTTP/1.0 200 OK\r\nA: b\r\n\r\n{\"x\":1}";
    assert_eq!(http::body_offset(resp), Some(25));
    assert_eq!(&resp[25..], b"{\"x\":1}");
    assert_eq!(http::body_offset(b"no separator"), None);
}
```

- [ ] **Step 2: Run → fails.**
- [ ] **Step 3: Implement.** `src/bufwrite.rs`:

```rust
//! core::fmt::Write over a fixed byte slice (replaces C snprintf).

use core::fmt;

pub struct SliceWriter<'a> {
    buf: &'a mut [u8],
    pos: usize,
}

impl<'a> SliceWriter<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        SliceWriter { buf, pos: 0 }
    }

    pub fn len(&self) -> usize {
        self.pos
    }

    pub fn is_empty(&self) -> bool {
        self.pos == 0
    }
}

impl fmt::Write for SliceWriter<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let bytes = s.as_bytes();
        if self.pos + bytes.len() > self.buf.len() {
            return Err(fmt::Error);
        }
        self.buf[self.pos..self.pos + bytes.len()].copy_from_slice(bytes);
        self.pos += bytes.len();
        Ok(())
    }
}
```

`src/http.rs`:

```rust
//! HTTP/1.0 request building and response splitting (port of src/net/http_util.c).

use crate::bufwrite::SliceWriter;
use core::fmt::Write;

pub fn build_request(dst: &mut [u8], host: &str, path: &str, body: &str) -> Option<usize> {
    let mut w = SliceWriter::new(dst);
    write!(
        w,
        "POST {path} HTTP/1.0\r\nHost: {host}\r\nContent-Type: application/json\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
    .ok()?;
    Some(w.len())
}

pub fn parse_status(resp: &[u8]) -> Option<u16> {
    if resp.len() < 12 || !resp.starts_with(b"HTTP/1.") {
        return None;
    }
    let mut status = 0u16;
    for &b in &resp[9..12] {
        if !b.is_ascii_digit() {
            return None;
        }
        status = status * 10 + (b - b'0') as u16;
    }
    Some(status)
}

pub fn body_offset(resp: &[u8]) -> Option<usize> {
    resp.windows(4).position(|w| w == b"\r\n\r\n").map(|i| i + 4)
}
```

- [ ] **Step 4: Gates pass.**
- [ ] **Step 5: Commit** — `feat(rust): port http request building and response splitting`

---

### Task 8: Time formatting and civil-date arithmetic

**Files:**
- Create: `rust/dashboard-core/src/time_fmt.rs`, `rust/dashboard-core/tests/time_fmt.rs`
- Modify: `lib.rs` (add `pub mod time_fmt;`)

**Interfaces:**
- Produces: `struct Date { year: i32, month: u8, day: u8 }` (Clone, Copy, PartialEq); `days_from_civil(Date) -> i64`; `civil_from_days(i64) -> Date` (needed by the firmware clock — C used `gmtime_r`); `weekday(Date) -> u8` (0 = Sunday, matches `tm_wday`); `banner(Date) -> heapless::String<16>` ("mer 26/08", French day names dim/lun/mar/mer/jeu/ven/sam); `hhmm(u8, u8) -> heapless::String<8>`; `ddmm(&str) -> heapless::String<8>` ("30/08", `"--"` fallback); `enum DeadlineClass { None, Overdue, Soon, Normal }`; `deadline_class(&str, Date) -> DeadlineClass`; `days_late(&str, Date) -> i64` (0 for empty/bad iso).

- [ ] **Step 1: Failing test `tests/time_fmt.rs`** (port of `test_time_fmt.c` + civil round-trip):

```rust
use dashboard_core::time_fmt::{self, Date, DeadlineClass};

fn d(y: i32, m: u8, day: u8) -> Date { Date { year: y, month: m, day } }

#[test]
fn banner_and_hhmm() {
    assert_eq!(time_fmt::banner(d(2026, 8, 26)).as_str(), "mer 26/08");
    assert_eq!(time_fmt::hhmm(14, 5).as_str(), "14:05");
}

#[test]
fn ddmm() {
    assert_eq!(time_fmt::ddmm("2026-08-30").as_str(), "30/08");
    assert_eq!(time_fmt::ddmm("").as_str(), "--");
}

#[test]
fn deadline_classes() {
    let today = d(2026, 8, 26);
    assert_eq!(time_fmt::deadline_class("2026-08-25", today), DeadlineClass::Overdue);
    assert_eq!(time_fmt::deadline_class("2026-08-26", today), DeadlineClass::Soon);
    assert_eq!(time_fmt::deadline_class("2026-08-27", today), DeadlineClass::Soon);
    assert_eq!(time_fmt::deadline_class("2026-08-28", today), DeadlineClass::Normal);
    assert_eq!(time_fmt::deadline_class("", today), DeadlineClass::None);
}

#[test]
fn month_rollover() {
    let today = d(2026, 8, 31);
    assert_eq!(time_fmt::deadline_class("2026-09-01", today), DeadlineClass::Soon);
    assert_eq!(time_fmt::deadline_class("2026-09-02", today), DeadlineClass::Normal);
}

#[test]
fn days_late() {
    let today = d(2026, 8, 26);
    assert_eq!(time_fmt::days_late("2026-08-24", today), 2);
    assert_eq!(time_fmt::days_late("2026-08-26", today), 0);
    assert_eq!(time_fmt::days_late("2026-08-28", today), -2);
    assert_eq!(time_fmt::days_late("2026-07-26", today), 31);
    assert_eq!(time_fmt::days_late("", today), 0);
}

#[test]
fn civil_round_trip() {
    assert_eq!(time_fmt::days_from_civil(d(1970, 1, 1)), 0);
    assert_eq!(time_fmt::weekday(d(1970, 1, 1)), 4);
    for days in [-1000i64, 0, 1, 20000, 20698, 30000] {
        let date = time_fmt::civil_from_days(days);
        assert_eq!(time_fmt::days_from_civil(date), days);
    }
    assert_eq!(time_fmt::civil_from_days(20698), d(2026, 8, 29));
}
```

- [ ] **Step 2: Run → fails.**
- [ ] **Step 3: Implement `src/time_fmt.rs`** (Hinnant civil algorithms replace mktime/gmtime — no DST, pure dates, which matches the data):

```rust
//! Date formatting and deadline classification (port of src/app/time_fmt.c,
//! with Hinnant civil-date arithmetic replacing mktime/gmtime).

use core::fmt::Write;
use heapless::String;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Date {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DeadlineClass { None, Overdue, Soon, Normal }

const DAY_NAMES: [&str; 7] = ["dim", "lun", "mar", "mer", "jeu", "ven", "sam"];

pub fn days_from_civil(d: Date) -> i64 {
    let y = i64::from(d.year) - i64::from(d.month <= 2);
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (i64::from(d.month) + 9) % 12;
    let doy = (153 * mp + 2) / 5 + i64::from(d.day) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

pub fn civil_from_days(z: i64) -> Date {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u8;
    let month = (if mp < 10 { mp + 3 } else { mp - 9 }) as u8;
    Date { year: (y + i64::from(month <= 2)) as i32, month, day }
}

pub fn weekday(d: Date) -> u8 {
    (days_from_civil(d) + 4).rem_euclid(7) as u8
}

pub fn banner(d: Date) -> String<16> {
    let mut s = String::new();
    let _ = write!(s, "{} {:02}/{:02}", DAY_NAMES[weekday(d) as usize], d.day, d.month);
    s
}

pub fn hhmm(hour: u8, minute: u8) -> String<8> {
    let mut s = String::new();
    let _ = write!(s, "{hour:02}:{minute:02}");
    s
}

pub fn ddmm(iso: &str) -> String<8> {
    let mut s = String::new();
    if iso.len() < 10 {
        let _ = s.push_str("--");
        return s;
    }
    let b = iso.as_bytes();
    let _ = write!(
        s,
        "{}{}/{}{}",
        b[8] as char, b[9] as char, b[5] as char, b[6] as char
    );
    s
}

fn parse_iso(iso: &str) -> Option<Date> {
    if iso.len() < 10 || iso.as_bytes()[4] != b'-' || iso.as_bytes()[7] != b'-' {
        return None;
    }
    Some(Date {
        year: iso[0..4].parse().ok()?,
        month: iso[5..7].parse().ok()?,
        day: iso[8..10].parse().ok()?,
    })
}

pub fn days_late(iso: &str, today: Date) -> i64 {
    match parse_iso(iso) {
        Some(due) => days_from_civil(today) - days_from_civil(due),
        None => 0,
    }
}

pub fn deadline_class(iso: &str, today: Date) -> DeadlineClass {
    if parse_iso(iso).is_none() {
        return DeadlineClass::None;
    }
    match days_late(iso, today) {
        late if late > 0 => DeadlineClass::Overdue,
        0 | -1 => DeadlineClass::Soon,
        _ => DeadlineClass::Normal,
    }
}
```

- [ ] **Step 4: Gates pass.**
- [ ] **Step 5: Commit** — `feat(rust): port time formatting on civil-date arithmetic`

---

### Task 9: Odoo model, request builders, response parsers

**Files:**
- Create: `rust/dashboard-core/src/odoo.rs`, `rust/dashboard-core/tests/odoo_request.rs`, `rust/dashboard-core/tests/odoo_parse.rs`
- Modify: `lib.rs` (add `pub mod odoo;`)

**Interfaces:**
- Produces:
  - `MAX_ACTIVITIES: usize = 7`, `FETCH_LIMIT: usize = 7`, `REQ_CAP: usize = 2048`, `MAX_JSON_TOKENS: usize = 512`
  - `struct Activity { name: heapless::String<64>, record: heapless::String<32>, deadline: heapless::String<11>, icon: heapless::String<24> }` (Default, Clone, PartialEq)
  - `struct ActivityList { count: u32, total: u32, items: [Activity; MAX_ACTIVITIES] }` (Default, Clone, PartialEq)
  - `struct Config<'a> { db, login, api_key, domain: &'a str }` (domain is the `%d` template from config)
  - `build_auth(dst: &mut [u8], cfg: &Config) -> Option<usize>`, `build_activities(dst, cfg, uid: i32) -> Option<usize>`, `build_count(dst, cfg, uid) -> Option<usize>` — byte-identical bodies to C
  - `enum ParseErr { Malformed, Odoo }`; `parse_auth(&[u8]) -> Result<i32, ParseErr>`, `parse_count(&[u8]) -> Result<u32, ParseErr>`, `parse_activities(&[u8], &mut ActivityList) -> Result<(), ParseErr>` (Odoo error object → `Err(Odoo)`).

- [ ] **Step 1: Failing tests.** `tests/odoo_request.rs` (port of `test_odoo_request.c`):

```rust
use dashboard_core::odoo::{self, Config};

const CFG: Config = Config {
    db: "testdb",
    login: "tester@test.lan",
    api_key: "test-key",
    domain: "[[\"user_id\",\"=\",%d]]",
};

fn s(buf: &[u8], len: usize) -> &str { std::str::from_utf8(&buf[..len]).unwrap() }

#[test]
fn auth_body() {
    let mut buf = [0u8; 512];
    let len = odoo::build_auth(&mut buf, &CFG).unwrap();
    let body = s(&buf, len);
    assert!(body.contains("\"service\":\"common\""));
    assert!(body.contains("\"authenticate\""));
    assert!(body.contains("\"testdb\""));
    assert!(body.contains("\"tester@test.lan\""));
    assert!(body.contains("\"test-key\""));
}

#[test]
fn activities_body() {
    let mut buf = [0u8; 2048];
    let len = odoo::build_activities(&mut buf, &CFG, 42).unwrap();
    let body = s(&buf, len);
    assert!(body.contains("\"execute_kw\""));
    assert!(body.contains("\"mail.activity\""));
    assert!(body.contains("\"search_read\""));
    assert!(body.contains("[[\"user_id\",\"=\",42]]"));
    assert!(body.contains("\"res_name\""));
    assert!(body.contains("\"summary\""));
    assert!(body.contains("\"icon\""));
    assert!(body.contains("\"limit\":7"));
    assert!(body.contains("\"order\":\"date_deadline asc\""));
}

#[test]
fn count_body() {
    let mut buf = [0u8; 2048];
    let len = odoo::build_count(&mut buf, &CFG, 42).unwrap();
    let body = s(&buf, len);
    assert!(body.contains("\"search_count\""));
    assert!(body.contains("[[\"user_id\",\"=\",42]]"));
    assert!(body.contains("\"id\":3"));
    assert!(!body.contains("\"limit\""));
}

#[test]
fn truncation() {
    let mut buf = [0u8; 32];
    assert!(odoo::build_auth(&mut buf, &CFG).is_none());
    assert!(odoo::build_activities(&mut buf, &CFG, 1).is_none());
    assert!(odoo::build_count(&mut buf, &CFG, 1).is_none());
}
```

`tests/odoo_parse.rs` (port of `test_odoo_parse.c`):

```rust
use dashboard_core::odoo::{self, ActivityList, ParseErr};

const AUTH: &[u8] = br#"{"jsonrpc":"2.0","id":1,"result":42}"#;
const AUTH_FAIL: &[u8] = br#"{"jsonrpc":"2.0","id":1,"result":false}"#;
const COUNT: &[u8] = br#"{"jsonrpc":"2.0","id":3,"result":23}"#;
const ERROR: &[u8] =
    br#"{"jsonrpc":"2.0","id":2,"error":{"code":200,"message":"Odoo Server Error"}}"#;
const ACTS: &[u8] = br#"{"jsonrpc":"2.0","id":2,"result":[
 {"id":7,"res_name":"Jean Dupont","summary":"Rappeler pour le devis",
  "icon":"fa-phone","date_deadline":"2026-08-30"},
 {"id":8,"res_name":"Facture F0042","summary":false,
  "icon":"fa-code","date_deadline":"2026-09-01"}]}"#;

#[test]
fn auth() {
    assert_eq!(odoo::parse_auth(AUTH), Ok(42));
    assert!(odoo::parse_auth(AUTH_FAIL).is_err());
    assert!(odoo::parse_auth(b"garbage").is_err());
}

#[test]
fn count() {
    assert_eq!(odoo::parse_count(COUNT), Ok(23));
    assert!(odoo::parse_count(AUTH_FAIL).is_err());
}

#[test]
fn activities_nominal() {
    let mut list = ActivityList::default();
    odoo::parse_activities(ACTS, &mut list).unwrap();
    assert_eq!(list.count, 2);
    assert_eq!(list.total, 0);
    assert_eq!(list.items[0].name.as_str(), "Rappeler pour le devis");
    assert_eq!(list.items[0].record.as_str(), "Jean Dupont");
    assert_eq!(list.items[0].icon.as_str(), "fa-phone");
    assert_eq!(list.items[0].deadline.as_str(), "2026-08-30");
}

#[test]
fn summary_fallback() {
    let mut list = ActivityList::default();
    odoo::parse_activities(ACTS, &mut list).unwrap();
    assert_eq!(list.items[1].name.as_str(), "Facture F0042");
    assert_eq!(list.items[1].record.as_str(), "");
    assert_eq!(list.items[1].icon.as_str(), "fa-code");
    assert_eq!(list.items[1].deadline.as_str(), "2026-09-01");
}

#[test]
fn error_and_garbage() {
    let mut list = ActivityList::default();
    assert_eq!(odoo::parse_activities(ERROR, &mut list), Err(ParseErr::Odoo));
    assert_eq!(
        odoo::parse_activities(b"not json at all", &mut list),
        Err(ParseErr::Malformed)
    );
}

#[test]
fn overflow_caps_at_seven() {
    let mut json = String::from("{\"result\":[");
    for i in 0..8 {
        if i > 0 { json.push(','); }
        json.push_str(&format!(
            "{{\"id\":{i},\"res_name\":\"T{i}\",\"date_deadline\":\"2026-09-01\"}}"
        ));
    }
    json.push_str("]}");
    let mut list = ActivityList::default();
    odoo::parse_activities(json.as_bytes(), &mut list).unwrap();
    assert_eq!(list.count, 7);
    assert_eq!(list.items[6].name.as_str(), "T6");
}
```

- [ ] **Step 2: Run → fails.**
- [ ] **Step 3: Implement `src/odoo.rs`**

```rust
//! Odoo activity model, JSON-RPC request builders and response parsers
//! (port of src/odoo/*.c).

use crate::bufwrite::SliceWriter;
use crate::json::{self, Ctx, TokKind, Token};
use core::fmt::Write;
use heapless::String;

pub const MAX_ACTIVITIES: usize = 7;
pub const FETCH_LIMIT: usize = MAX_ACTIVITIES;
pub const REQ_CAP: usize = 2048;
pub const MAX_JSON_TOKENS: usize = 512;

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct Activity {
    pub name: String<64>,
    pub record: String<32>,
    pub deadline: String<11>,
    pub icon: String<24>,
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct ActivityList {
    pub count: u32,
    pub total: u32,
    pub items: [Activity; MAX_ACTIVITIES],
}

pub struct Config<'a> {
    pub db: &'a str,
    pub login: &'a str,
    pub api_key: &'a str,
    pub domain: &'a str,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ParseErr { Malformed, Odoo }

fn write_domain(w: &mut SliceWriter, tmpl: &str, uid: i32) -> Option<()> {
    match tmpl.split_once("%d") {
        Some((pre, post)) => write!(w, "{pre}{uid}{post}").ok(),
        None => w.write_str(tmpl).ok(),
    }
}

pub fn build_auth(dst: &mut [u8], cfg: &Config) -> Option<usize> {
    let mut w = SliceWriter::new(dst);
    write!(
        w,
        "{{\"jsonrpc\":\"2.0\",\"method\":\"call\",\"params\":{{\"service\":\"common\",\
         \"method\":\"authenticate\",\"args\":[\"{}\",\"{}\",\"{}\",{{}}]}},\"id\":1}}",
        cfg.db, cfg.login, cfg.api_key
    )
    .ok()?;
    Some(w.len())
}

pub fn build_activities(dst: &mut [u8], cfg: &Config, uid: i32) -> Option<usize> {
    let mut w = SliceWriter::new(dst);
    write!(
        w,
        "{{\"jsonrpc\":\"2.0\",\"method\":\"call\",\"params\":{{\"service\":\"object\",\
         \"method\":\"execute_kw\",\"args\":[\"{}\",{},\"{}\",\"mail.activity\",\
         \"search_read\",[",
        cfg.db, uid, cfg.api_key
    )
    .ok()?;
    write_domain(&mut w, cfg.domain, uid)?;
    write!(
        w,
        "],{{\"fields\":[\"res_name\",\"summary\",\"icon\",\"date_deadline\"],\
         \"limit\":{FETCH_LIMIT},\"order\":\"date_deadline asc\"}}]}},\"id\":2}}"
    )
    .ok()?;
    Some(w.len())
}

pub fn build_count(dst: &mut [u8], cfg: &Config, uid: i32) -> Option<usize> {
    let mut w = SliceWriter::new(dst);
    write!(
        w,
        "{{\"jsonrpc\":\"2.0\",\"method\":\"call\",\"params\":{{\"service\":\"object\",\
         \"method\":\"execute_kw\",\"args\":[\"{}\",{},\"{}\",\"mail.activity\",\
         \"search_count\",[",
        cfg.db, uid, cfg.api_key
    )
    .ok()?;
    write_domain(&mut w, cfg.domain, uid)?;
    write!(w, "]]}},\"id\":3}}").ok()?;
    Some(w.len())
}

fn fold_to<const N: usize>(src: &[u8]) -> String<N> {
    let mut buf = [0u8; N];
    let len = json::fold_into(&mut buf, src);
    let mut s = String::new();
    if let Ok(text) = core::str::from_utf8(&buf[..len]) {
        let _ = s.push_str(text);
    }
    s
}

fn parse_result_int(json_bytes: &[u8]) -> Result<i64, ParseErr> {
    let mut toks = [Token::EMPTY; 64];
    let count = json::parse(json_bytes, &mut toks).map_err(|()| ParseErr::Malformed)?;
    let ctx = Ctx { json: json_bytes, toks: &toks[..count] };
    let val = ctx.find_key("result").ok_or(ParseErr::Malformed)?;
    if ctx.toks[val].kind != TokKind::Primitive {
        return Err(ParseErr::Malformed);
    }
    let text = core::str::from_utf8(ctx.bytes(val)).map_err(|_| ParseErr::Malformed)?;
    if !text.as_bytes().first().is_some_and(u8::is_ascii_digit) {
        return Err(ParseErr::Malformed);
    }
    text.parse().map_err(|_| ParseErr::Malformed)
}

pub fn parse_auth(json_bytes: &[u8]) -> Result<i32, ParseErr> {
    Ok(parse_result_int(json_bytes)? as i32)
}

pub fn parse_count(json_bytes: &[u8]) -> Result<u32, ParseErr> {
    Ok(parse_result_int(json_bytes)? as u32)
}

fn set_field(ctx: &Ctx, key: usize, act: &mut Activity) {
    let val = key + 1;
    if val >= ctx.toks.len() || ctx.toks[val].kind != TokKind::Str {
        return;
    }
    if ctx.tok_eq(key, "summary") {
        act.name = fold_to(ctx.bytes(val));
    } else if ctx.tok_eq(key, "res_name") {
        act.record = fold_to(ctx.bytes(val));
    } else if ctx.tok_eq(key, "date_deadline") {
        act.deadline = fold_to(ctx.bytes(val));
    } else if ctx.tok_eq(key, "icon") {
        act.icon = fold_to(ctx.bytes(val));
    }
}

fn parse_activity(ctx: &Ctx, idx: usize, act: &mut Activity) {
    let mut child = idx + 1;
    for _ in 0..ctx.toks[idx].size {
        set_field(ctx, child, act);
        child = ctx.next_sibling(child + 1);
    }
    if act.name.is_empty() {
        act.name = String::new();
        let _ = act.name.push_str(act.record.as_str());
        act.record.clear();
    }
}

pub fn parse_activities(json_bytes: &[u8], list: &mut ActivityList) -> Result<(), ParseErr> {
    let mut toks = [Token::EMPTY; MAX_JSON_TOKENS];
    *list = ActivityList::default();
    let count = json::parse(json_bytes, &mut toks).map_err(|()| ParseErr::Malformed)?;
    let ctx = Ctx { json: json_bytes, toks: &toks[..count] };
    if ctx.find_key("error").is_some() {
        return Err(ParseErr::Odoo);
    }
    let val = ctx.find_key("result").ok_or(ParseErr::Malformed)?;
    if ctx.toks[val].kind != TokKind::Array {
        return Err(ParseErr::Malformed);
    }
    let total = ctx.toks[val].size;
    let mut child = val + 1;
    for n in 0..total {
        if n < MAX_ACTIVITIES {
            parse_activity(&ctx, child, &mut list.items[n]);
        }
        child = ctx.next_sibling(child);
    }
    list.count = total.min(MAX_ACTIVITIES) as u32;
    Ok(())
}
```

Note the name-size subtlety preserved from C: `name` capacity 64 vs `record` capacity 32 — the fallback copy always fits.

- [ ] **Step 4: Gates pass.**
- [ ] **Step 5: Commit** — `feat(rust): port odoo model, request builders and parsers`

---

### Task 10: Activity-icon mapping and refresh policy

**Files:**
- Create: `rust/dashboard-core/src/activity_icon.rs`, `rust/dashboard-core/src/refresh.rs`, `rust/dashboard-core/tests/activity_icon.rs`, `rust/dashboard-core/tests/refresh.rs`
- Modify: `lib.rs` (add both mods)

**Interfaces:**
- Produces: `activity_icon::icon_for(&str) -> Icon` (exact-match table, `Icon::Clock` fallback); `refresh::{Snapshot { list: ActivityList, offline: bool }, RefreshTimes { now_s: u32, last_refresh_s: u32, has_displayed: bool }, MIN_GAP_S: u32 = 180, MAX_AGE_S: u32 = 86_400, refresh_needed(&Snapshot, &Snapshot, &RefreshTimes) -> bool}`. `Snapshot: Default + PartialEq` (derive replaces C memcmp — contents-equal semantics).

- [ ] **Step 1: Failing tests.** `tests/activity_icon.rs` (port):

```rust
use dashboard_core::activity_icon::icon_for;
use dashboard_core::icon_data::Icon;

#[test]
fn known_classes() {
    assert_eq!(icon_for("fa-check"), Icon::Check);
    assert_eq!(icon_for("fa-envelope"), Icon::Envelope);
    assert_eq!(icon_for("fa-phone"), Icon::Phone);
    assert_eq!(icon_for("fa-users"), Icon::Users);
    assert_eq!(icon_for("fa-code"), Icon::Code);
    assert_eq!(icon_for("fa-cube"), Icon::Cube);
    assert_eq!(icon_for("fa-microchip"), Icon::Microchip);
    assert_eq!(icon_for("fa-lightbulb-o"), Icon::Lightbulb);
    assert_eq!(icon_for("fa-calendar-check-o"), Icon::CalendarCheck);
    assert_eq!(icon_for("fa-upload"), Icon::Upload);
    assert_eq!(icon_for("fa-pencil-square-o"), Icon::PenSquare);
}

#[test]
fn unknown_falls_back() {
    assert_eq!(icon_for("fa-rocket"), Icon::Clock);
    assert_eq!(icon_for(""), Icon::Clock);
    assert_eq!(icon_for("fa-che"), Icon::Clock);
}
```

`tests/refresh.rs` (port of `test_refresh.c`):

```rust
use dashboard_core::refresh::{refresh_needed, RefreshTimes, Snapshot};

#[test]
fn first_display() {
    let a = Snapshot::default();
    let t = RefreshTimes { now_s: 30, last_refresh_s: 0, has_displayed: false };
    assert!(refresh_needed(&a, &a, &t));
}

#[test]
fn unchanged_skips() {
    let a = Snapshot::default();
    let t = RefreshTimes { now_s: 10_000, last_refresh_s: 5_000, has_displayed: true };
    assert!(!refresh_needed(&a, &a, &t));
}

#[test]
fn change_triggers_when_spaced() {
    let a = Snapshot::default();
    let mut b = Snapshot::default();
    b.list.count = 1;
    let _ = b.list.items[0].name.push_str("X");
    let spaced = RefreshTimes { now_s: 1_000, last_refresh_s: 500, has_displayed: true };
    let close = RefreshTimes { now_s: 600, last_refresh_s: 500, has_displayed: true };
    assert!(refresh_needed(&a, &b, &spaced));
    assert!(!refresh_needed(&a, &b, &close));
}

#[test]
fn daily_health_refresh() {
    let a = Snapshot::default();
    let t = RefreshTimes { now_s: 90_000, last_refresh_s: 100, has_displayed: true };
    assert!(refresh_needed(&a, &a, &t));
}

#[test]
fn offline_flip_triggers() {
    let a = Snapshot::default();
    let mut b = Snapshot::default();
    b.offline = true;
    let t = RefreshTimes { now_s: 1_000, last_refresh_s: 500, has_displayed: true };
    assert!(refresh_needed(&a, &b, &t));
}
```

- [ ] **Step 2: Run → fails.**
- [ ] **Step 3: Implement.** `src/activity_icon.rs`:

```rust
//! Mapping from Odoo activity icon classes to embedded icons
//! (port of src/app/activity_icon.c).

use crate::icon_data::Icon;

const RULES: &[(&str, Icon)] = &[
    ("fa-check", Icon::Check),
    ("fa-envelope", Icon::Envelope),
    ("fa-phone", Icon::Phone),
    ("fa-users", Icon::Users),
    ("fa-inbox", Icon::Inbox),
    ("fa-code", Icon::Code),
    ("fa-cube", Icon::Cube),
    ("fa-microchip", Icon::Microchip),
    ("fa-lightbulb-o", Icon::Lightbulb),
    ("fa-calendar-check-o", Icon::CalendarCheck),
    ("fa-calendar", Icon::CalendarCheck),
    ("fa-upload", Icon::Upload),
    ("fa-pencil-square-o", Icon::PenSquare),
    ("fa-pencil", Icon::PenSquare),
    ("fa-tasks", Icon::Check),
    ("fa-clock-o", Icon::Clock),
];

pub fn icon_for(class: &str) -> Icon {
    RULES
        .iter()
        .find(|(needle, _)| *needle == class)
        .map_or(Icon::Clock, |(_, ic)| *ic)
}
```

`src/refresh.rs`:

```rust
//! Refresh-on-change decision policy (port of src/app/refresh.c).

use crate::odoo::ActivityList;

pub const MIN_GAP_S: u32 = 180;
pub const MAX_AGE_S: u32 = 86_400;

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct Snapshot {
    pub list: ActivityList,
    pub offline: bool,
}

pub struct RefreshTimes {
    pub now_s: u32,
    pub last_refresh_s: u32,
    pub has_displayed: bool,
}

pub fn refresh_needed(prev: &Snapshot, cur: &Snapshot, t: &RefreshTimes) -> bool {
    if !t.has_displayed {
        return true;
    }
    let age = t.now_s.wrapping_sub(t.last_refresh_s);
    let changed = prev != cur;
    (changed || age >= MAX_AGE_S) && age >= MIN_GAP_S
}
```

- [ ] **Step 4: Gates pass.**
- [ ] **Step 5: Commit** — `feat(rust): port icon mapping and refresh policy`

---

### Task 11: Dashboard rendering

**Files:**
- Create: `rust/dashboard-core/src/dashboard.rs`, `rust/dashboard-core/tests/dashboard.rs`
- Modify: `lib.rs` (add `pub mod dashboard;`)

**Interfaces:**
- Consumes: gfx, text, time_fmt, activity_icon, refresh.
- Produces: constants `BANNER_H=72, ROWS_TOP=80, ROW_H=48, FOOTER_Y=444, MARGIN=16, GAP=24, TEXT_X=60` (all `i32`); `struct DashboardData<'a> { snap: &'a Snapshot, today: Date, banner_date: &'a str, updated_hhmm: &'a str, offline_since: &'a str }`; `render(fb: &mut Frame, d: &DashboardData)`. Pixel-for-pixel port of `src/app/dashboard.c` (title "Planning", date at `MARGIN + text_width("Planning",3) + 40`, "{total} activites" right-aligned, icon at MARGIN, deadline right-pinned (Red only when overdue), red "{days}j" chip 12px left of the date, name at TEXT_X with "..." ellipsis, record right-aligned in leftover span, 2px full-width separators, footer "mise a jour {hhmm}" / "HORS LIGNE depuis {t}" / "+ d'autres activites ({n})", empty state "Aucune activite" green scale 3 at y 210).

- [ ] **Step 1: Failing test `tests/dashboard.rs`** (port of `test_dashboard.c`):

```rust
use dashboard_core::dashboard::{self, DashboardData};
use dashboard_core::gfx::{Color, Frame, Rect, BUFFER_SIZE, WIDTH};
use dashboard_core::refresh::Snapshot;
use dashboard_core::time_fmt::Date;

fn px(fb: &Frame, x: i32, y: i32) -> u8 {
    let idx = (y as usize * WIDTH as usize + x as usize) / 2;
    if x & 1 == 0 { fb[idx] >> 4 } else { fb[idx] & 0x0f }
}

fn region_has(fb: &Frame, r: &Rect, c: Color) -> bool {
    for y in r.y..r.y + r.h {
        for x in r.x..r.x + r.w {
            if px(fb, x, y) == c as u8 { return true; }
        }
    }
    false
}

fn data<'a>(snap: &'a Snapshot) -> DashboardData<'a> {
    DashboardData {
        snap,
        today: Date { year: 2026, month: 8, day: 26 },
        banner_date: "mer 26/08",
        updated_hhmm: "14:35",
        offline_since: "",
    }
}

fn new_frame() -> Box<Frame> { Box::new([0u8; BUFFER_SIZE]) }

#[test]
fn banner_and_rows() {
    let mut snap = Snapshot::default();
    snap.list.count = 1;
    snap.list.total = 1;
    let _ = snap.list.items[0].name.push_str("Activite urgente");
    let _ = snap.list.items[0].deadline.push_str("2026-08-20");
    let _ = snap.list.items[0].icon.push_str("fa-phone");
    let mut fb = new_frame();
    dashboard::render(&mut fb, &data(&snap));
    assert_eq!(px(&fb, 5, 5), Color::Green as u8);
    assert!(region_has(&fb, &Rect { x: 16, y: 80, w: 300, h: 48 }, Color::Black));
    assert!(region_has(&fb, &Rect { x: WIDTH - 160, y: 80, w: 160, h: 48 }, Color::Red));
}

#[test]
fn empty_state() {
    let snap = Snapshot::default();
    let mut fb = new_frame();
    dashboard::render(&mut fb, &data(&snap));
    assert!(region_has(&fb, &Rect { x: 0, y: 200, w: WIDTH, h: 120 }, Color::Green));
}

#[test]
fn offline_footer() {
    let mut snap = Snapshot::default();
    snap.offline = true;
    let mut fb = new_frame();
    let mut d = data(&snap);
    d.offline_since = "14:00";
    dashboard::render(&mut fb, &d);
    assert!(region_has(&fb, &Rect { x: 0, y: 440, w: WIDTH, h: 40 }, Color::Red));
}

#[test]
fn overflow_footer() {
    let mut snap = Snapshot::default();
    snap.list.count = 1;
    snap.list.total = 4;
    let _ = snap.list.items[0].name.push_str("Une activite");
    let mut fb = new_frame();
    dashboard::render(&mut fb, &data(&snap));
    assert!(region_has(&fb, &Rect { x: 0, y: 440, w: 400, h: 40 }, Color::Black));
}

#[test]
fn deterministic() {
    let mut snap = Snapshot::default();
    snap.list.count = 1;
    snap.list.total = 1;
    let _ = snap.list.items[0].name.push_str("Stable");
    let mut a = new_frame();
    let mut b = new_frame();
    dashboard::render(&mut a, &data(&snap));
    dashboard::render(&mut b, &data(&snap));
    assert_eq!(a[..], b[..]);
}
```

- [ ] **Step 2: Run → fails.**
- [ ] **Step 3: Implement `src/dashboard.rs`**

```rust
//! Dashboard layout rendering (port of src/app/dashboard.c).

use crate::activity_icon::icon_for;
use crate::gfx::{self, Color, Frame, Rect, Style, HEIGHT as _, WIDTH};
use crate::odoo::Activity;
use crate::refresh::Snapshot;
use crate::text;
use crate::time_fmt::{self, Date, DeadlineClass};
use core::fmt::Write;
use heapless::String;

pub const BANNER_H: i32 = 72;
pub const ROWS_TOP: i32 = 80;
pub const ROW_H: i32 = 48;
pub const FOOTER_Y: i32 = 444;
pub const MARGIN: i32 = 16;
pub const GAP: i32 = 24;
pub const TEXT_X: i32 = 60;

pub struct DashboardData<'a> {
    pub snap: &'a Snapshot,
    pub today: Date,
    pub banner_date: &'a str,
    pub updated_hhmm: &'a str,
    pub offline_since: &'a str,
}

fn draw_banner(fb: &mut Frame, d: &DashboardData) {
    gfx::fill_rect(fb, &Rect { x: 0, y: 0, w: WIDTH, h: BANNER_H }, Color::Green);
    let title = Style { x: MARGIN, y: 12, color: Color::White, scale: 3 };
    text::text(fb, &title, "Planning");
    let date_x = MARGIN + text::text_width("Planning", 3) + 40;
    text::text(fb, &Style { x: date_x, y: 24, color: Color::White, scale: 2 }, d.banner_date);
    let mut counter: String<24> = String::new();
    let _ = write!(counter, "{} activites", d.snap.list.total);
    let cx = WIDTH - MARGIN - text::text_width(&counter, 2);
    text::text(fb, &Style { x: cx, y: 24, color: Color::White, scale: 2 }, &counter);
}

fn deadline_color(act: &Activity, today: Date) -> Color {
    if time_fmt::deadline_class(&act.deadline, today) == DeadlineClass::Overdue {
        Color::Red
    } else {
        Color::Black
    }
}

fn draw_name(fb: &mut Frame, name: &str, y: i32, max_w: i32) -> Option<i32> {
    let st = Style { x: TEXT_X, y, color: Color::Black, scale: 2 };
    let fit = text::text_fit(name, 2, max_w);
    if fit == name.len() {
        text::text(fb, &st, name);
        return Some(st.x + text::text_width(name, 2));
    }
    let fit = text::text_fit(name, 2, max_w - text::text_width("...", 2));
    let mut line: String<72> = String::new();
    let _ = line.push_str(&name[..fit]);
    let _ = line.push_str("...");
    text::text(fb, &st, &line);
    None
}

fn draw_late(fb: &mut Frame, act: &Activity, today: Date, y: i32, to: i32) -> i32 {
    let days = time_fmt::days_late(&act.deadline, today);
    if days <= 0 {
        return to;
    }
    let mut chip: String<16> = String::new();
    let _ = write!(chip, "{days}j");
    let x = to - 12 - text::text_width(&chip, 2);
    text::text(fb, &Style { x, y, color: Color::Red, scale: 2 }, &chip);
    x
}

fn draw_extra(fb: &mut Frame, act: &Activity, y: i32, from: i32, to: i32) {
    let width = text::text_width(&act.record, 2);
    if act.record.is_empty() || width > to - from {
        return;
    }
    text::text(fb, &Style { x: to - width, y, color: Color::Black, scale: 2 }, &act.record);
}

fn draw_row(fb: &mut Frame, act: &Activity, y: i32, today: Date) {
    let ty = y + 8;
    text::icon(
        fb,
        &Style { x: MARGIN, y: ty, color: Color::Black, scale: 2 },
        icon_for(&act.icon),
    );
    let ddmm = time_fmt::ddmm(&act.deadline);
    let date_x = WIDTH - MARGIN - text::text_width(&ddmm, 2);
    text::text(
        fb,
        &Style { x: date_x, y: ty, color: deadline_color(act, today), scale: 2 },
        &ddmm,
    );
    let to = draw_late(fb, act, today, ty, date_x);
    let Some(name_end) = draw_name(fb, &act.name, ty, to - GAP - TEXT_X) else {
        return;
    };
    let from = name_end + GAP;
    let to = to - GAP;
    if to > from {
        draw_extra(fb, act, ty, from, to);
    }
}

fn draw_rows(fb: &mut Frame, d: &DashboardData) {
    let mut y = ROWS_TOP;
    for i in 0..d.snap.list.count as usize {
        draw_row(fb, &d.snap.list.items[i], y, d.today);
        gfx::fill_rect(fb, &Rect { x: 0, y: y + ROW_H - 2, w: WIDTH, h: 2 }, Color::Black);
        y += ROW_H;
    }
}

fn draw_footer(fb: &mut Frame, d: &DashboardData) {
    let mut line: String<48> = String::new();
    let _ = write!(line, "mise a jour {}", d.updated_hhmm);
    let rx = WIDTH - 8 - text::text_width(&line, 2);
    text::text(fb, &Style { x: rx, y: FOOTER_Y, color: Color::Black, scale: 2 }, &line);
    if d.snap.offline {
        let mut msg: String<48> = String::new();
        let _ = write!(msg, "HORS LIGNE depuis {}", d.offline_since);
        text::text(fb, &Style { x: 8, y: FOOTER_Y, color: Color::Red, scale: 2 }, &msg);
        return;
    }
    if d.snap.list.total > d.snap.list.count {
        let mut msg: String<48> = String::new();
        let _ = write!(msg, "+ d'autres activites ({})", d.snap.list.total - d.snap.list.count);
        text::text(fb, &Style { x: 8, y: FOOTER_Y, color: Color::Black, scale: 2 }, &msg);
    }
}

pub fn render(fb: &mut Frame, d: &DashboardData) {
    gfx::fill(fb, Color::White);
    draw_banner(fb, d);
    if d.snap.list.count == 0 {
        text::text_centered(
            fb,
            &Style { x: 0, y: 210, color: Color::Green, scale: 3 },
            "Aucune activite",
        );
    } else {
        draw_rows(fb, d);
    }
    draw_footer(fb, d);
}
```

(If clippy rejects the unused `HEIGHT as _` import, drop it.)

- [ ] **Step 4: Gates pass.**
- [ ] **Step 5: Commit** — `feat(rust): port dashboard layout rendering`

---

### Task 12: Preview tool, C fixture, frame-parity test  ← Phase 1 parity gate for rendering

**Files:**
- Create: `rust/host-tools/src/lib.rs` (replace stub), `rust/host-tools/src/bin/preview.rs` (replace stub), `rust/host-tools/tests/parity.rs`, `rust/host-tools/tests/fixtures/preview_c.raw` (binary, generated from the C tool)

**Interfaces:**
- Produces: `host_tools::render_sample(fb: &mut Frame)` building the EXACT scenario of `tests/render_preview.c` (6 activities, total 11, today 2026-08-27, banner "jeu 27/08", updated "03:15"); `preview` bin writing `preview.raw`; the parity test.

- [ ] **Step 1: Generate the C fixture** (PowerShell, usual preamble):

```powershell
make -C tests CC=clang preview.exe
Set-Location tests; .\preview.exe; Set-Location ..
Copy-Item tests\preview.raw rust\host-tools\tests\fixtures\preview_c.raw
(Get-Item rust\host-tools\tests\fixtures\preview_c.raw).Length   # must print 192000
```

- [ ] **Step 2: `src/lib.rs`**

```rust
//! Host-side sample-frame rendering shared by the preview bin and parity test.

use dashboard_core::dashboard::{self, DashboardData};
use dashboard_core::gfx::Frame;
use dashboard_core::odoo::Activity;
use dashboard_core::refresh::Snapshot;
use dashboard_core::time_fmt::Date;

fn act(name: &str, record: &str, deadline: &str, icon: &str) -> Activity {
    let mut a = Activity::default();
    let _ = a.name.push_str(name);
    let _ = a.record.push_str(record);
    let _ = a.deadline.push_str(deadline);
    let _ = a.icon.push_str(icon);
    a
}

pub fn sample_snapshot() -> Snapshot {
    let mut snap = Snapshot::default();
    snap.list.count = 6;
    snap.list.total = 11;
    snap.list.items[0] = act("Rappeler pour le devis", "Jean Dupont", "2026-08-25", "fa-phone");
    snap.list.items[1] = act(
        "Relire et valider la proposition commerciale complete",
        "Societe Martin & Fils",
        "2026-08-27",
        "fa-code",
    );
    snap.list.items[2] = act("Facture F0042", "", "2026-08-19", "fa-cube");
    snap.list.items[3] = act("Envoyer le contrat signe", "Dossier 2318", "2026-09-02", "fa-microchip");
    snap.list.items[4] = act("Point hebdo equipe", "", "2026-08-28", "fa-lightbulb-o");
    snap.list.items[5] = act("Preparer la demo", "Projet Alpha", "2026-09-05", "fa-calendar-check-o");
    snap
}

pub fn render_sample(fb: &mut Frame) {
    let snap = sample_snapshot();
    let d = DashboardData {
        snap: &snap,
        today: Date { year: 2026, month: 8, day: 27 },
        banner_date: "jeu 27/08",
        updated_hhmm: "03:15",
        offline_since: "",
    };
    dashboard::render(fb, &d);
}
```

- [ ] **Step 3: `src/bin/preview.rs`**

```rust
use dashboard_core::gfx::{Frame, BUFFER_SIZE};

fn main() -> std::io::Result<()> {
    let mut fb: Box<Frame> = Box::new([0u8; BUFFER_SIZE]);
    host_tools::render_sample(&mut fb);
    std::fs::write("preview.raw", &fb[..])?;
    println!("preview: wrote preview.raw");
    Ok(())
}
```

- [ ] **Step 4: `tests/parity.rs`**

```rust
use dashboard_core::gfx::{Frame, BUFFER_SIZE};

#[test]
fn frame_matches_c_renderer_byte_for_byte() {
    let fixture: &[u8] = include_bytes!("fixtures/preview_c.raw");
    assert_eq!(fixture.len(), BUFFER_SIZE);
    let mut fb: Box<Frame> = Box::new([0u8; BUFFER_SIZE]);
    host_tools::render_sample(&mut fb);
    if fb[..] != *fixture {
        let first = fb.iter().zip(fixture).position(|(a, b)| a != b).unwrap();
        let (x, y) = ((first % 400) * 2, first / 400);
        panic!(
            "frame differs from C render at byte {first} (approx pixel x={x}, y={y}): rust=0x{:02x} c=0x{:02x}",
            fb[first], fixture[first]
        );
    }
}
```

- [ ] **Step 5: Run `cargo test --workspace`** — the parity test MUST pass. If it fails, the panic message localizes the first divergent pixel; debug the Rust port (the C fixture is ground truth) — common causes are glyph advance arithmetic, banner date x offset, and the day-chip 12px gap. Do not regenerate the fixture to make the test pass.
- [ ] **Step 6: Optional visual check** — `cargo run -p host-tools --bin preview` then reuse the existing python raw→PNG snippet from the C workflow if a human-viewable image is wanted.
- [ ] **Step 7: Gates + commit** — `feat(rust): byte-exact frame parity against the c renderer`

---

### Task 13: Firmware crate scaffold (boots, logs over USB, UF2)

**Files:**
- Create: `rust/dashboard-firmware/Cargo.toml`, `rust/dashboard-firmware/build.rs`, `rust/dashboard-firmware/memory.x`, `rust/.cargo/config.toml`, `rust/dashboard-firmware/src/main.rs` (skeleton), `rust/dashboard-firmware/src/config.rs.example`
- Modify: `rust/Cargo.toml` (add member `dashboard-firmware` — NOT to default-members)

**Interfaces:**
- Produces: flashable UF2 that prints `epaper_dashboard: toolchain OK` once per second over USB serial; `config.rs` convention (`include!`-free plain module, gitignored, same names as C: `WIFI_SSID`, `WIFI_PASSWORD`, `WIFI_COUNTRY: &str`, `ODOO_HOST`, `ODOO_PORT: u16`, `ODOO_DB`, `ODOO_LOGIN`, `ODOO_API_KEY`, `ODOO_ACTIVITY_DOMAIN`, `POLL_INTERVAL_S: u32`, `NTP_SERVER`, `TZ_OFFSET_MIN: i32`, `EPD_CLK_HALF_PERIOD_US: u32`, `ODOO_CA_CERT: Option<&str>`).

This task carries the ecosystem friction; the reconciliation clause applies. Use the embassy repo's `examples/rp235x` as the API reference (crates.io docs or the vendored sources under `%USERPROFILE%\.cargo\registry\src`).

- [ ] **Step 1: `rust/.cargo/config.toml`**

```toml
[target.thumbv8m.main-none-eabihf]
rustflags = ["-C", "link-arg=--nmagic", "-C", "link-arg=-Tlink.x"]

[env]
DEFMT_LOG = "off"
```

- [ ] **Step 2: `memory.x`** (RP2350, from the embassy rp235x examples):

```
MEMORY {
    FLASH : ORIGIN = 0x10000000, LENGTH = 4096K
    RAM   : ORIGIN = 0x20000000, LENGTH = 512K
}

SECTIONS {
    .start_block : ALIGN(4) {
        __start_block_addr = .;
        KEEP(*(.start_block));
        KEEP(*(.boot_info));
    } > FLASH
} INSERT AFTER .vector_table;

_stext = ADDR(.start_block) + SIZEOF(.start_block);

SECTIONS {
    .bi_entries : ALIGN(4) {
        __bi_entries_start = .;
        KEEP(*(.bi_entries));
        . = ALIGN(4);
        __bi_entries_end = .;
    } > FLASH
} INSERT AFTER .text;

SECTIONS {
    .end_block : ALIGN(4) {
        __end_block_addr = .;
        KEEP(*(.end_block));
    } > FLASH
} INSERT AFTER .uninit;

PROVIDE(start_to_end = __end_block_addr - __start_block_addr);
PROVIDE(end_to_start = __start_block_addr - __end_block_addr);
```

`build.rs`: copy the canonical cortex-m-rt build.rs (put `memory.x` on the linker search path and `println!("cargo:rerun-if-changed=memory.x")`).

- [ ] **Step 3: `Cargo.toml`** — package `dashboard-firmware`, `edition 2021`, dependencies per Global Constraints pins plus `dashboard-core = { path = "../dashboard-core" }`, `panic-halt` NOT used (own handler later; for the skeleton a minimal `#[panic_handler]` looping is fine).
- [ ] **Step 4: `src/main.rs` skeleton** (reference shape — reconcile against the pinned embassy-rp rp235x example):

```rust
//! e-paper dashboard firmware (Embassy, RP2350).
#![no_std]
#![no_main]

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
    spawner.spawn(logger_task(Driver::new(p.USB, Irqs))).unwrap();
    loop {
        log::info!("epaper_dashboard: toolchain OK");
        Timer::after_secs(1).await;
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
```

**Log-string requirement:** the USB output line must read exactly `epaper_dashboard: toolchain OK`. If `embassy-usb-logger` prefixes a level tag, use its custom-formatter hook to emit the bare message (or replace with a minimal CDC-ACM writer task); report which was needed.

- [ ] **Step 5: `src/config.rs.example`**

```rust
//! Copy to src/config.rs (gitignored) and fill in real values.
#![allow(dead_code)]

pub const WIFI_SSID: &str = "MyNetwork";
pub const WIFI_PASSWORD: &str = "MyPassword";
pub const WIFI_COUNTRY: &str = "FR";
pub const ODOO_HOST: &str = "odoo.example.com";
pub const ODOO_PORT: u16 = 443;
pub const ODOO_DB: &str = "mydb";
pub const ODOO_LOGIN: &str = "me@example.com";
pub const ODOO_API_KEY: &str = "paste-api-key-here";
pub const ODOO_ACTIVITY_DOMAIN: &str = "[[\"user_id\",\"=\",%d]]";
pub const POLL_INTERVAL_S: u32 = 300;
pub const NTP_SERVER: &str = "pool.ntp.org";
pub const TZ_OFFSET_MIN: i32 = 120;
pub const EPD_CLK_HALF_PERIOD_US: u32 = 1;
pub const ODOO_CA_CERT: Option<&str> = None;
```

Create the local gitignored `src/config.rs` from it (dummy values fine for compiling; NEVER commit it — verify with `git status`).

- [ ] **Step 6: Build + UF2**

```powershell
cargo build --release -p dashboard-firmware --target thumbv8m.main-none-eabihf
& "C:\Users\charl\picotool-install\bin\picotool.exe" uf2 convert `
  target\thumbv8m.main-none-eabihf\release\dashboard-firmware -t elf `
  target\thumbv8m.main-none-eabihf\release\dashboard-firmware.uf2
```

- [ ] **Step 7 [HW] (optional):** offer the user a flash to confirm `toolchain OK` on serial; continue regardless.
- [ ] **Step 8: Gates + commit** — `feat(rust): embassy firmware skeleton boots and logs over usb`

---

### Task 14: Panel driver port

**Files:**
- Create: `rust/dashboard-firmware/src/epd.rs`
- Modify: `src/main.rs` (add `mod epd;` + `mod config;`)

**Interfaces:**
- Consumes: embassy-rp GPIO, embassy-time, `config::EPD_CLK_HALF_PERIOD_US`.
- Produces: `struct Epd` owning pins RST=12, DC=8, CS=9, BUSY=13(pull-up in), CLK=10, DIN=11; `Epd::new(...pins...) -> Self`, `fn init(&mut self, wdt: &mut Watchdog) -> Result<(), ()>` (3 reset+init attempts, power-on-reaction check), `fn display(&mut self, fb: &Frame, wdt: &mut Watchdog) -> Result<(), ()>` (0x10 + buffer, then 0x04/0x12/0x02 each followed by busy-wait), `fn sleep(&mut self)` (0x07 0xA5). Port the exact 13-command init sequence, reset timing 1(20ms)/0(2ms)/1(20ms), BUSY low = busy, timeouts (short 5s, long 60s, reaction window 3s), watchdog fed every 256 bytes and in wait loops. Bit pacing: `embassy_time::block_for(Duration::from_micros(EPD_CLK_HALF_PERIOD_US as u64))` before and between clock edges, mirroring C.

- [ ] **Step 1: Implement `epd.rs`** — direct transcription of `src/epd/epd_io.c` + `src/epd/epd_driver.c` into one module (init sequence as `const INIT_SEQUENCE: &[(u8, &[u8])]` with the exact 13 entries: `(0xAA,[0x49,0x55,0x20,0x08,0x09,0x18]), (0x01,[0x3F]), (0x00,[0x5F,0x69]), (0x03,[0x00,0x54,0x00,0x44]), (0x05,[0x40,0x1F,0x1F,0x2C]), (0x06,[0x6F,0x1F,0x17,0x49]), (0x08,[0x6F,0x1F,0x1F,0x22]), (0x30,[0x03]), (0x50,[0x3F]), (0x60,[0x02,0x00]), (0x61,[0x03,0x20,0x01,0xE0]), (0x84,[0x01]), (0xE3,[0x2F])`). Verify byte-for-byte against `src/epd/epd_driver.c` before committing; report any mismatch instead of changing either side.
- [ ] **Step 2: Wire a temporary smoke call** in main (construct Epd; do NOT refresh the panel — just `init` is forbidden standalone too; compile-only here). Keep main's loop logging.
- [ ] **Step 3: Build + UF2 + gates.**
- [ ] **Step 4: Commit** — `feat(rust): port paced bit-bang e-paper driver`

---

### Task 15: WiFi bring-up `[HW]`

**Files:**
- Create: `rust/dashboard-firmware/src/net.rs`, `rust/dashboard-firmware/cyw43-firmware/43439A0.bin`, `rust/dashboard-firmware/cyw43-firmware/43439A0_clm.bin`, `rust/dashboard-firmware/cyw43-firmware/LICENSE-permissive-binary-license-1.0.txt`
- Modify: `src/main.rs`

**Interfaces:**
- Produces: `net::init(spawner, p_pins...) -> (embassy_net::Stack, cyw43::Control)` — brings up cyw43 (PIO SPI on PIN_23/24/25/29 + PIO0 + DMA_CH0, standard Pico W wiring), spawns the cyw43 runner and embassy-net runner tasks, DHCP config; `net::join(control, stack) -> bool` joining WPA2 with `config::WIFI_SSID/WIFI_PASSWORD` and waiting for DHCP (`stack.wait_config_up()`), 30 s timeout. Boot loop in main: retry with backoff 5 s doubling to 80 s cap, logging exactly `wifi: retry in {} ms` / `wifi: connected`.

- [ ] **Step 1: Vendor the cyw43 firmware blobs** from the embassy repo (`https://github.com/embassy-rs/embassy/raw/main/cyw43-firmware/...`: `43439A0.bin`, `43439A0_clm.bin`, plus the license file from the same directory). `include_bytes!` them.
- [ ] **Step 2: Implement `net.rs`** following the pinned embassy `rp235x` wifi example (cyw43::new → spawn runner → `control.init(clm)` → `set_power_management(PowerManagementMode::PowerSave)` → join via `control.join(SSID, JoinOptions::new(PASSWORD.as_bytes()))`; embassy-net `Stack` with `Config::dhcpv4(Default::default())`, `StackResources<8>` in `StaticCell`, random seed from `embassy_rp::clocks::RoscRng`). Reconciliation clause applies.
- [ ] **Step 3: main boot sequence** — after the logger task: `wifi_boot()` mirroring C (`net::join` in a loop with backoff; log strings exact).
- [ ] **Step 4: Build + UF2; [HW]:** user flashes, serial shows `wifi: connected`. Continue regardless.
- [ ] **Step 5: Gates + commit** — `feat(rust): cyw43 wifi bring-up with dhcp and backoff`

---

### Task 16: SNTP and wall clock `[HW]`

**Files:**
- Create: `rust/dashboard-firmware/src/sntp.rs`, `rust/dashboard-firmware/src/clock.rs`
- Modify: `src/main.rs`

**Interfaces:**
- Produces: `sntp::query(stack) -> Option<u32>` (UDP to `config::NTP_SERVER:123`, DNS-resolved; 48-byte request, first byte 0x1B; reply transmit-timestamp seconds at offset 40..44 big-endian minus 2_208_988_800 → unix secs; 10 s timeout); `clock::Clock` storing `unix_at_boot: u32` anchored to `embassy_time::Instant`; `clock.now_unix() -> u32`, `clock.now_local(&self) -> (Date, u8, u8)` (applies `TZ_OFFSET_MIN`, uses `time_fmt::civil_from_days` for Y/M/D and div/rem for hh:mm), `clock.uptime_s() -> u32` (drives refresh arithmetic — C `now_s()` equivalent). Boot: `time_boot()` loops `sntp::query` every 500 ms until success, then logs exactly `time: synced`.

- [ ] **Step 1: Implement both modules** (UdpSocket from embassy-net with 2 small buffers; parse reply defensively: length ≥ 44, mode bits sane).
- [ ] **Step 2: Wire into main** after wifi; log line exact.
- [ ] **Step 3: Build + UF2; [HW]:** serial shows `wifi: connected` then `time: synced`. Gates + commit — `feat(rust): sntp time sync anchored to the monotonic clock`

---

### Task 17: TLS HTTP client

**Files:**
- Create: `rust/dashboard-firmware/src/net_http.rs`
- Modify: `src/main.rs` (mod decl only)

**Interfaces:**
- Consumes: `dashboard_core::http`, embassy-net TCP + DNS, embedded-tls, `embassy_rp::trng::Trng` (RNG for the handshake; fall back to `RoscRng` if Trng is unavailable in the pinned embassy-rp — report which).
- Produces: `net_http::post_json(stack, rng, path: &str, body: &str, resp_buf: &'a mut [u8; 16384]) -> Result<Response<'a>, NetErr>` where `Response { status: u16, body: &'a [u8] }` (NUL-free; body slice into resp_buf); `enum NetErr { Dns, Connect, Tls, Io, Http }`. Behavior: DNS resolve (10 s), TCP connect (15 s), TLS 1.3 open with SNI = `ODOO_HOST` and **no cert verification** when `ODOO_CA_CERT.is_none()` (embedded-tls `NoVerify`/`UnsecureProvider`), write request built by `http::build_request`, read to EOF or buffer full (30 s total), parse via `http::parse_status` + `http::body_offset`, require status 200. One fresh connection per call (`Connection: close`). TLS record buffers: two 16.5 KB static buffers via `StaticCell`.

- [ ] **Step 1: Implement** per the embedded-tls README example for embassy-net (`TlsConfig::new().with_server_name(host)`, `TlsConnection::new(socket, read_buf, write_buf)`, `open::<_, NoVerify>` / `UnsecureProvider::new::<Aes128GcmSha256>(rng)` depending on the pinned API — reconciliation clause). Pinned-CA mode: when `ODOO_CA_CERT` is `Some(pem)`, pass it as the TLS config's CA (embedded-tls `Certificate::X509`/webpki feature); if the pinned embedded-tls cannot verify against a custom CA, log-and-proceed unverified and report the limitation prominently (C's default posture was unverified too).
- [ ] **Step 2: Build + UF2 + gates.**
- [ ] **Step 3: Commit** — `feat(rust): https post client over embedded-tls`

---

### Task 18: Odoo client, superloop main, panic log — full firmware `[HW]` end-to-end

**Files:**
- Create: `rust/dashboard-firmware/src/odoo_client.rs`, `rust/dashboard-firmware/src/panic_log.rs`
- Modify: `rust/dashboard-firmware/src/main.rs` (full replacement of the demo loop)

**Interfaces:**
- Produces:
  - `odoo_client::sync(stack, rng, uid: &mut i32, list: &mut ActivityList) -> bool` — auth when `*uid <= 0` (log exactly `odoo: authenticated uid={}`), fetch activities, re-auth once on `ParseErr::Odoo`, then `search_count` into `list.total` (clamped `>= list.count`); any failure → false.
  - `panic_log`: `#[panic_handler]` writing the panic message (`core::fmt::Write` of `PanicInfo`) into a `#[link_section = ".uninit.PANIC"] static` 160-byte buffer + magic `0x50414e43`, then watchdog-reboot; `panic_log::report()` on boot prints `last panic: {msg}` once and clears the magic.
  - `main`: boot = usb logger → 2 s delay → `panic_log::report()` → watchdog start (8 s; feed in all loops) → net init → epd pins init → `wifi_boot()` → `time_boot()` → superloop every `POLL_INTERVAL_S`: sync → `note_result` (success logs `odoo: {} activities`; failure logs `odoo: fetch failed ({} in a row)`, restores displayed list into current, 3rd consecutive failure sets offline + `offline_since` hh:mm) → `refresh_needed` → on refresh: build `DashboardData` (banner/hhmm from `clock.now_local()`), `dashboard::render`, log `epd: refresh start`, `epd.init` + `epd.display` + `epd.sleep` (sleep ALWAYS once init succeeded), log `epd: refresh done`/`epd: refresh failed`, update displayed/last_refresh/has_displayed only on success. The 187.5 KB framebuffer and both snapshots live in `StaticCell`/`static` — never on a task stack.
- Log strings (byte-exact C set): `wifi: retry in {} ms`, `wifi: connected`, `time: synced`, `odoo: authenticated uid={}`, `odoo: {} activities`, `odoo: fetch failed ({} in a row)`, `epd: refresh start`, `epd: refresh done`, `epd: refresh failed`, `epd: panel not responding`, `last panic: {}`.

- [ ] **Step 1: Implement the three files** (port `src/odoo/odoo_client.c` + `src/app/main.c` + `src/app/panic_log.c` logic onto the async primitives from T13–T17).
- [ ] **Step 2: Build + UF2 + all gates.**
- [ ] **Step 3 [HW] end-to-end validation with the user** (real `config.rs`): staged checklist — 1. `wifi: connected` 2. `time: synced` 3. `odoo: authenticated uid=N` 4. `odoo: N activities` 5. `epd: refresh start` → ~35 s flash → `epd: refresh done`, panel shows the same dashboard as the C build 6. steady-state polls print counts with no `epd:` lines when nothing changed 7. change an activity in Odoo → refresh on next poll 8. router unplugged → 3 failures → offline footer; recovery works 9. `git status` shows `config.rs` untracked.
- [ ] **Step 4: Commit** — `feat(rust): full dashboard firmware with superloop and panic log`

---

### Task 19: Rust build documentation

**Files:**
- Create: `rust/README.md`
- Modify: `README.md` (one short pointer section "Rust rewrite (feat/rust-rewrite)" near the top)

- [ ] **Step 1: Write `rust/README.md`**: toolchain install (rustup + target), config setup (`cp src/config.rs.example src/config.rs` in dashboard-firmware), host tests (`cargo test --workspace`), parity test explanation + fixture regeneration recipe (C preview → copy → only when the LAYOUT intentionally changes in BOTH implementations), firmware build + picotool UF2 command + BOOTSEL flash, log expectations, the reconciliation/pinning policy. Base every command on what actually worked in T1–T18 reports.
- [ ] **Step 2: Gates (docs don't break builds) + commit** — `docs(rust): build, test and flash instructions`

---

### Task 20: Phase 2 — baseline measurements and size tuning

**Files:**
- Create: `rust/MEASUREMENTS.md`
- Modify: `rust/Cargo.toml` (release profile)

- [ ] **Step 1: Record baseline** in `rust/MEASUREMENTS.md` as a table with these rows, measured now:
  - C UF2 size: `(Get-Item build\epaper_dashboard.uf2).Length` (reference value 1,092,096 bytes)
  - Rust UF2 size (pre-tuning)
  - Static RAM: `& "$env:PICO_TOOLCHAIN_PATH\arm-none-eabi-size.exe" target\thumbv8m.main-none-eabihf\release\dashboard-firmware` (text/data/bss) — same command against `build\epaper_dashboard.elf` for C
  - Poll-cycle duration: from a serial log, seconds between `epd: refresh start` and `epd: refresh done`, and between consecutive poll log lines (reuse the T18 [HW] transcript; mark "n/a" if unavailable).
- [ ] **Step 2: Tune the release profile** in the workspace `Cargo.toml`:

```toml
[profile.release]
opt-level = "z"
lto = "fat"
codegen-units = 1
panic = "abort"
```

- [ ] **Step 3: Rebuild, re-run ALL gates (parity included), re-measure, append the post-tuning column.**
- [ ] **Step 4: Commit** — `perf(rust): size-tuned release profile with measured baseline`

---

### Task 21: Phase 2 — idiomatic refactor sweep

**Files:**
- Modify: `rust/dashboard-core/src/*.rs`, `rust/dashboard-firmware/src/*.rs` as listed.

Concrete refactors (each keeps every test — parity included — green; run the full gate suite after each bullet):

- [ ] **Step 1:** enable `#![warn(clippy::pedantic)]` in `dashboard-core/src/lib.rs`; fix every lint or add a one-line-justified `#[allow]` (justifications like "C-parity arithmetic" are valid).
- [ ] **Step 2:** `gfx::fill_rect` row-wise nibble writes for full-width rects (the separator/banner hot path): compute byte spans per row and use slice `fill` when the rect is byte-aligned; fall back to `pixel` at odd edges. Parity test proves equivalence.
- [ ] **Step 3:** `text::draw_glyph` inner loop → iterate `rows.chunks(stride)` with `enumerate`, hoist the color/nibble computation.
- [ ] **Step 4:** replace remaining `as` numeric casts in core with `i32::from`/`u32::from`/`try_into` where lossless (keep `as` only in documented truncation spots).
- [ ] **Step 5:** extract row-geometry arithmetic in `dashboard.rs` (date_x / chip span / name span) into small pure functions; add direct unit tests for them in `tests/dashboard.rs` asserting the exact values the parity frame implies.
- [ ] **Step 6:** re-measure (same table columns) and append an "after idiomatic" column to `rust/MEASUREMENTS.md`.
- [ ] **Step 7: Commit** — `refactor(rust): idiomatic core with parity pinned`

---

### Task 22: Promotion — Rust becomes the firmware

Only after the user confirms the T18 hardware validation (and ideally a re-flash after T20/21).

**Files:**
- Delete: `src/`, `include/`, `tests/` (C), `CMakeLists.txt`, `pico_sdk_import.cmake`, `lwipopts.h`, `mbedtls_config.h`, `config.h.example`, `tools/check_style.py`
- Move: `rust/*` → repo root (`Cargo.toml`, `rust-toolchain.toml`, `.cargo/`, `dashboard-core/`, `dashboard-firmware/`, `host-tools/`, `MEASUREMENTS.md`); keep `tools/`, `lib/`, `hardware/`, `docs/`, `DRIVER_REFERENCE.md`, `micropython/`.
- Modify: `.gitignore` (drop C entries — `build/`, `tests/*.exe`, `include/config.h`, `tests/preview.raw`; keep/adjust `target/`, `dashboard-firmware/src/config.rs`), `.gitattributes` (drop C globs), `tools/gen_font.py` + `tools/gen_icons.py` (remove C emitters; Rust output paths updated to the new locations), `README.md` (full rewrite for the Rust toolchain: hardware section and panel safety preserved verbatim; toolchain = rustup section; config = config.rs; tests = cargo test; build/flash = cargo + picotool; troubleshooting updated; measurements table referenced).

- [ ] **Step 1: Execute moves with `git mv`** (history preserved), delete with `git rm`.
- [ ] **Step 2: Fix paths** — generator output paths, parity-fixture `include_bytes!` is crate-relative (unchanged), `.cargo/config.toml` now at root; run everything.
- [ ] **Step 3: Full gates** — `cargo fmt --check`, clippy, `cargo test --workspace` (parity green), firmware build + UF2, `python tools/gen_font.py lib/questrial/Questrial-Regular.ttf && git diff --exit-code` (regeneration idempotent).
- [ ] **Step 4: Commit** — `feat!: promote rust firmware, retire the c implementation`
- [ ] **Step 5:** leave merging `feat/rust-rewrite` → `main` to the finishing-a-development-branch flow with the user.

---

## Plan self-review (done at authoring time)

- **Spec coverage:** §1 behavioral parity → log strings frozen (T13/15/16/18), request bytes (T9 tests), frame bytes (T12); §2 approach → T3–T12 ports, pins in Global Constraints; §3 workspace → T2/T13, generators → T4, style gates → Global Constraints; §4 network/time → T15–T17, SNTP → T16, RAM posture → static buffers throughout (T18 note); §5 display → T14 (exact init bytes listed), layout → T11; §6 verification → 11 suites mapped to tests/{json_fold,odoo_parse,odoo_request,time_fmt,http,refresh,gfx,text,dashboard,activity_icon}.rs + parity (T12) + [HW] (T18); §7 Phase 2 → T20–T21 with measured table, promotion → T22; §8 out-of-scope respected (no new features anywhere).
- **Placeholder scan:** firmware tasks (T13–T17) intentionally carry reference-plus-reconcile instructions tied to pinned crates — sanctioned by the spec's ecosystem reality and bounded by frozen interfaces/log strings; no TBDs elsewhere; all core code and tests are complete.
- **Type consistency:** `Frame`/`Color`/`Style` (T3) used by T5/T11/T12; `Icon` variant order (T4) matches `activity_icon` (T10) and `ICONS` list in gen_icons.py; `ActivityList{count,total,items}` (T9) matches refresh (T10), dashboard (T11), sample (T12), odoo_client (T18); `fold_into` dst-size semantics (T6) match heapless capacities 64/32/11/24 (T9); `http::build_request(host,…)` param order consistent T7/T17.
