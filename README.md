# Odoo e-Paper Dashboard — Pico 2 W, C firmware

Bare-metal C rewrite (Pico SDK, no OS, no dynamic allocation in project
code) of a MicroPython prototype. A Raspberry Pi Pico 2 W joins your WiFi,
authenticates against a self-hosted Odoo instance over HTTPS, and shows
the signed-in user's open project tasks on a Waveshare 7.3" e-Paper HAT
(E) — Spectra 6, 800x480, 6 fixed colors (black, white, yellow, red,
blue, green).

It polls Odoo's JSON-RPC API every 5 minutes but only spends a physical
refresh cycle on the panel when the task list actually changed, or once
a day for a mandatory health refresh. Spectra 6 panels have no
partial-refresh mode, so minimizing wear means minimizing *refreshes*,
not minimizing pixels touched. Everything runs from a single superloop
with static buffers; the only dynamic allocation anywhere is mbedTLS's
internal handshake scratch space, freed right after each TLS handshake.

Screen layout (800x480):

```
┌──────────────────────────────────────────────────────────────┐
│  MES TACHES                    mar 26/08         12 ouvertes │  banner 60px:
├──────────────────────────────────────────────────────────────┤  BLUE bg, WHITE text
│  * Corriger module facture    Compta      26/08    En cours  │
│    Banniere site web          Site        28/08    A faire   │  12 rows x 32px
│    ...                                                       │
├──────────────────────────────────────────────────────────────┤
│  +3 autres                             mise a jour 14:35     │  footer 24px
└──────────────────────────────────────────────────────────────┘
```

A red `*` marks a high-priority task; the deadline column turns red when
overdue, blue when due today or tomorrow. The footer shows the timestamp
of the last *screen update*, not a live clock (a ticking clock would
force a refresh every minute, defeating the whole point).

## 1. Hardware and wiring

- MCU: Raspberry Pi Pico 2 W (RP2350, 520 KB SRAM, CYW43439 WiFi). The
  "2 WH" (pre-soldered headers) variant is identical firmware-wise.
- Panel: Waveshare 7.3" e-Paper HAT (E), Spectra 6, 800x480.
- Bus: 4-wire SPI, but driven as bit-banged GPIO on purpose — hardware
  SPI at >= 1 MHz corrupts commands on this wiring (Dupont wires +
  ribbon cable). Do not "optimize" this back to hardware SPI without
  re-validating on real hardware; see `DRIVER_REFERENCE.md` section 2.
- **SPI Select switch on the HAT must be on 0.**

| Signal | Pico pin | Constant   |
|--------|----------|------------|
| VCC    | 3V3 OUT (pin 36) | — |
| GND    | GND      | —          |
| DIN    | GP11     | `DIN_PIN`  |
| CLK    | GP10     | `CLK_PIN`  |
| CS     | GP9      | `CS_PIN`   |
| DC     | GP8      | `DC_PIN`   |
| RST    | GP12     | `RST_PIN`  |
| BUSY   | GP13     | `BUSY_PIN` |

Panel safety rules, all enforced in the firmware but worth knowing:

- `epd_sleep()` is called after every refresh, on every code path
  including failures. Leaving the panel powered outside sleep mode
  holds it at high internal voltage and causes irreversible damage.
- At least 3 minutes between refreshes (`REFRESH_MIN_GAP_S = 180`).
- At least one refresh every 24 hours while powered
  (`REFRESH_MAX_AGE_S = 86400`).
- One `epd_display()` per fully-composed frame — never partial sends.

## 2. Toolchain setup, from zero

These are the packages the firmware actually needs to cross-compile for
the Pico 2 W (PowerShell, run as your normal user — no admin needed for
winget user-scope installs):

```powershell
winget install -e --id Kitware.CMake --accept-source-agreements --accept-package-agreements
winget install -e --id Ninja-build.Ninja --accept-package-agreements
winget install -e --id Arm.GnuArmEmbeddedToolchain --accept-package-agreements
```

Clone the Pico SDK (2.3.0 or any later stable 2.x tag) with its four
required submodules — the SDK itself is much bigger than these four, but
only these are needed for this project's WiFi/lwIP/mbedTLS/USB stack:

```powershell
git clone --depth 1 --branch 2.3.0 https://github.com/raspberrypi/pico-sdk.git $env:USERPROFILE\pico-sdk
cd $env:USERPROFILE\pico-sdk
git submodule update --init --depth 1 lib/cyw43-driver lib/lwip lib/mbedtls lib/tinyusb
```

Point the SDK at the ARM toolchain and the checkout (adjust the version
folder in the ARM path to whatever winget actually installed —
`Get-ChildItem 'C:\Program Files (x86)\Arm GNU Toolchain arm-none-eabi'`
will show it):

```powershell
[Environment]::SetEnvironmentVariable('PICO_SDK_PATH', "$env:USERPROFILE\pico-sdk", 'User')
[Environment]::SetEnvironmentVariable('PICO_TOOLCHAIN_PATH', 'C:\Program Files (x86)\Arm GNU Toolchain arm-none-eabi\14.2 rel1\bin', 'User')
```

Open a new terminal (persisted User environment variables are not
picked up by already-running shells), then verify:

```powershell
cmake --version
ninja --version
arm-none-eabi-gcc --version
```

Note: an earlier draft of this setup called for a bundled host
toolchain ("w64devkit") alongside the ARM one. That package does not
exist in the winget catalog, and the firmware itself never needs a host
compiler — CMake cross-compiles everything with the ARM toolchain
above. A host C compiler is only needed for two things, both covered
below: running the unit tests under `tests/`, and (rarely) building
`picotool` from source during the very first `cmake --build`.

### Host compiler for tests

`tests/` builds and runs on your PC, not the Pico, so it needs a plain
host C compiler and `make`. Any working gcc or clang is fine. On this
project's development machine, the obvious winget substitute for a host
gcc (`BrechtSanders.WinLibs.POSIX.UCRT`) turned out to have a bundled
assembler (`as.exe`) that Windows Smart App Control silently blocks
when it's invoked as a subprocess — which breaks every build that
depends on it, tests included, with a confusing "CreateProcess: No such
file or directory" error that has nothing to do with the actual code.
`clang` was installed as a working substitute instead:

```powershell
winget install -e --id LLVM.LLVM --accept-package-agreements
```

and tests are run with `CC=clang` (see "Host tests" below). If your
system gcc already works cleanly, plain `make -C tests run` is fine —
there is nothing gcc-specific about the test code.

## 3. Configuration

Copy the template and fill in real values — `include/config.h` is
gitignored from the first commit, so credentials can never be committed
by accident:

```powershell
cp config.h.example include/config.h
```

Fields, in the order they appear in `config.h.example`:

| Macro | Meaning |
|---|---|
| `WIFI_SSID` / `WIFI_PASSWORD` | Your WiFi network credentials. |
| `WIFI_COUNTRY` | A `CYW43_COUNTRY_*` constant from the Pico SDK (e.g. `CYW43_COUNTRY_FRANCE`); affects the legal WiFi channel set. |
| `ODOO_HOST` | Odoo hostname only — no `https://`, no path. |
| `ODOO_PORT` | HTTPS port, normally `443`. |
| `ODOO_DB` | Odoo database name. |
| `ODOO_LOGIN` | Your Odoo login (usually an email address). |
| `ODOO_API_KEY` | An Odoo API key (not your account password — see below). |
| `ODOO_TASK_DOMAIN` | A JSON-RPC domain filter, as a C string with `%d` where the authenticated `uid` goes (see below). |
| `POLL_INTERVAL_S` | Seconds between Odoo polls, default 300 (5 minutes). |
| `NTP_SERVER` | SNTP server for time sync, e.g. `pool.ntp.org`. |
| `TZ_OFFSET_MIN` | Local timezone offset from UTC, in minutes (no DST database — a plain fixed offset, adjust by hand across DST changes). |
| `EPD_CLK_HALF_PERIOD_US` | Bit-bang clock half-period in microseconds. Default `1` (~500 kHz effective). Do not lower this without re-validating on real hardware — see the SPI corruption warning in section 1. |

`ODOO_CA_CERT` is not in the template's macro list; it is documented as
an optional addition in a comment at the bottom of
`config.h.example` — see "Optional: pinning the Odoo CA certificate"
below.

**Keep `"` out of every config value.** All of these are compiled as C
string literals; an unescaped `"` inside a password, login, or hostname
will break the build (or silently truncate the value at best).

### Creating an Odoo API key

Use an API key, not your login password: in Odoo, click your user
avatar (top right) -> **My Profile** -> **Account Security** tab ->
**New API Key**. Give it a description, confirm, and copy the key
immediately — Odoo only shows it once.

### Tuning `ODOO_TASK_DOMAIN`

The domain is a JSON-RPC filter, `printf`-substituted at request-build
time: `%d` is replaced with the `uid` returned by `authenticate`. The
default,

```c
#define ODOO_TASK_DOMAIN "[[\"user_ids\",\"in\",[%d]],[\"is_closed\",\"=\",false]]"
```

fetches tasks assigned to the current user whose stage isn't a "closed"
(folded) stage. If your Odoo version's `project.task` model doesn't
have `is_closed` (older or heavily customized instances), edit this one
line — for example, filter on the stage's own `fold` flag instead:

```c
#define ODOO_TASK_DOMAIN "[[\"user_ids\",\"in\",[%d]],[\"stage_id.fold\",\"=\",false]]"
```

or drop the second clause entirely to show every assigned task
regardless of stage. No firmware code needs to change for this — it's
a config-only edit.

## 4. Optional: pinning the Odoo CA certificate

By default `ODOO_CA_CERT` is undefined, which means TLS still encrypts
the connection but does **not** verify the server's identity — a LAN
man-in-the-middle could intercept the connection and read your Odoo API
key. To pin the CA and get full certificate verification, fetch the
certificate chain:

```
openssl s_client -connect ODOO_HOST:443 -showcerts
```

(run from Git Bash or WSL if `openssl` isn't on your Windows PATH).
Take the **last** `-----BEGIN CERTIFICATE-----` ... `-----END
CERTIFICATE-----` block printed (the top of the chain, i.e. the issuing
CA), and turn it into a single C string with explicit `\n` line breaks:

```c
#define ODOO_CA_CERT "-----BEGIN CERTIFICATE-----\n" \
    "MIIB...\n" \
    "...\n" \
    "-----END CERTIFICATE-----\n"
```

Add this to your local `include/config.h`. That file is gitignored and
is explicitly exempted from the project's single-line-macro style rule
for exactly this reason (see the comment in `config.h.example`) — a
multi-line PEM string is fine there.

## 5. Build and flash

```powershell
cmake -S . -B build -G Ninja
cmake --build build
```

The build fails fast with a clear `#error` if `include/config.h` is
missing (see section 3). The first configure also downloads and builds
`picotool` from source — needed to turn the linked ELF into
`.uf2`/`.bin`/`.hex` for RP2350 — which requires a working **native**
host C/C++ compiler (not the ARM cross toolchain). If that host
compiler is broken (this project hit exactly the Smart App
Control/`as.exe` issue described in section 2 here too, since picotool
also tried to use the WinLibs GCC install), build `picotool` once with
any other working host compiler (e.g. MSVC's `cl.exe`, if Visual Studio
is installed) and point CMake at the prebuilt install instead of
letting it build from source:

```powershell
cmake -S build/_deps/picotool-src -B <picotool-build-dir> -G Ninja `
    -DCMAKE_C_COMPILER=cl -DCMAKE_CXX_COMPILER=cl -DPICOTOOL_NO_LIBUSB=1 `
    -DPICO_SDK_PATH=$env:PICO_SDK_PATH `
    -DCMAKE_INSTALL_PREFIX=<picotool-install-dir> -DCMAKE_BUILD_TYPE=Release
cmake --build <picotool-build-dir> --config Release
cmake --install <picotool-build-dir> --config Release
Remove-Item -Recurse -Force build
cmake -S . -B build -G Ninja -DCMAKE_PREFIX_PATH="<picotool-install-dir>"
```

The configure log will then say `Using picotool from
<picotool-install-dir>/bin/picotool.exe` instead of building it from
source, and `cmake --build build` proceeds normally.

To flash: hold **BOOTSEL**, plug the Pico 2 W into USB, release BOOTSEL
once it shows up as a mass-storage drive (`RP2350`), then drag-and-drop
`build/epaper_dashboard.uf2` onto that drive. The board reboots and
starts running the firmware automatically.

Open a serial monitor at **115200 baud** on the Pico's USB serial port
to watch the staged boot log (WiFi connect, SNTP sync, Odoo
authenticate, task count, refresh decisions).

## 6. Host tests

```
make -C tests run
```

builds and runs the ten host-side unit test binaries (`test_harness`,
`test_json_str`, `test_odoo_parse`, `test_odoo_request`,
`test_time_fmt`, `test_http_util`, `test_refresh`, `test_gfx`,
`test_gfx_text`, `test_dashboard`) — plain C, no Pico SDK dependency,
covering `odoo_parse` against canned JSON (nominal tasks, empty list,
Odoo error objects, truncated JSON, oversized fields, accented text),
the refresh-decision matrix, graphics primitives, and dashboard layout.

On this project's development machine, plain `make -C tests run` fails
for the same Smart App Control reason described in section 2 (the
WinLibs GCC substitute's assembler is blocked), so tests were actually
run with:

```
make -C tests CC=clang run
```

Expect every line to read `<name>: OK` and the command to exit 0.

## 7. Behavior reference

- After boot (WiFi connect with exponential backoff, 5 s doubling to an
  80 s cap; SNTP time sync; one Odoo `authenticate` call caching the
  `uid`), the firmware polls Odoo's JSON-RPC API every
  `POLL_INTERVAL_S` seconds (default 300 = 5 minutes).
- The panel is refreshed only when the fetched task snapshot actually
  changed, or at least once per 24 hours, and never less than 3 minutes
  after the previous refresh — otherwise the display is left untouched
  and the panel takes zero wear.
- Three consecutive failed polls count as a state change: the firmware
  forces one refresh showing an offline footer (`HORS LIGNE depuis
  HH:MM`, in red). The next successful poll forces a refresh back to
  the normal view.
- A hardware watchdog (8 s) reboots the board if any code path stops
  feeding it — the WiFi connect/retry loop, the inter-poll idle wait,
  and the panel's busy-wait loop all feed it periodically, so normal
  operation never trips it; only a genuine hang does.
- If the WiFi chip itself fails to initialize at boot (before any SSID
  or password is even used), the onboard LED fast-blinks (on/off every
  100 ms) forever and the firmware never proceeds further. That pattern
  always means the WiFi chip did not come up — not a wrong SSID or
  password, which instead shows as an endless "wifi: retry" backoff in
  the serial log.
- The dashboard renders ASCII only: the vendored font has no accented
  glyphs, so French accented characters coming back from Odoo (é, è,
  à, ç, ...) are folded to their unaccented equivalent while parsing the
  JSON response; any other multi-byte UTF-8 sequence becomes `?`.

## 8. Troubleshooting

- **Panel completely unresponsive** (serial log shows "epd: panel not
  responding", or a boot-time `RuntimeError` in the MicroPython
  reference): almost always a loose Dupont wire on CLK/DIN/DC/CS, or
  the SPI Select switch not on 0. Full diagnostic order in
  `DRIVER_REFERENCE.md` section 6.
- **Text renders mirrored on the physical panel**: `src/gfx_text.c`
  reads each `font8x8` row LSB-leftmost (`(bits >> col) & 1`). If your
  panel behaves as though the bit order is reversed, change that one
  line to `(bits >> (7 - col)) & 1` — a deliberately isolated one-line
  fix, left as a note from the original port rather than baked in
  speculatively.
- **Build fails with `include/config.h missing`**: see section 3 — copy
  `config.h.example` there and fill it in.
- **LED fast-blinks forever right after power-on**: WiFi chip init
  failure (see "Behavior reference" above) — check the board/antenna,
  this is not a credentials problem.
- **Offline footer shows even though WiFi is clearly up**: check
  `ODOO_HOST`/`ODOO_PORT`/`ODOO_DB`/`ODOO_LOGIN`/`ODOO_API_KEY`, and
  whether `ODOO_TASK_DOMAIN` matches your Odoo version's `project.task`
  fields (see "Tuning `ODOO_TASK_DOMAIN`" in section 3).
