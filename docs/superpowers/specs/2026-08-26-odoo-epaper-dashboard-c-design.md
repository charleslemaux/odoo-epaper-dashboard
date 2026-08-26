# Odoo e-Paper Dashboard in C — Design

Date: 2026-08-26
Status: approved design, pending implementation plan

## 1. Goal

Rewrite the MicroPython e-paper dashboard as a bare-metal C firmware for the
Raspberry Pi Pico 2 W, so that RAM usage is minimal and the device runs
standalone: connect to WiFi, poll a self-hosted Odoo instance over HTTPS every
5 minutes for the user's open tasks, and show them on the Waveshare 7.3"
e-Paper HAT (E). The panel is only refreshed when the task data actually
changed (plus one mandatory daily health refresh) — Spectra 6 panels do not
support partial refresh, so "render only what's needed" is implemented as
"refresh only when needed".

All project code follows the Epitech C Coding Style (v7): snake_case, 80
columns, function bodies <= 20 lines, <= 4 parameters, <= 5 exported and
<= 10 total functions per file, no comments inside function bodies, Epitech
file headers, include guards, no goto, static for file-local symbols.
The only exemptions are vendored third-party code (`lib/`) and generated
files.

## 2. Hardware & fixed constraints

- MCU: Raspberry Pi Pico 2 W (RP2350, 520 KB SRAM, 4 MB flash, CYW43439
  WiFi). The "2 WH" variant is identical firmware-wise.
- Panel: Waveshare 7.3" e-Paper HAT (E), Spectra 6, 800x480, 6 fixed colors
  (BLACK 0x0, WHITE 0x1, YELLOW 0x2, RED 0x3, BLUE 0x5, GREEN 0x6),
  4 bits/pixel GS4_HMSB framebuffer = 192 000 bytes.
- Wiring (unchanged, SPI Select switch on 0): DIN=GP11, CLK=GP10, CS=GP9,
  DC=GP8, RST=GP12, BUSY=GP13, VCC=3V3, GND=GND.
- Transport: bit-bang GPIO. Hardware SPI >= 1 MHz corrupts commands on this
  wiring (validated finding — do not reintroduce hardware SPI without
  on-device revalidation). In C the bit-bang loop must be *paced down*:
  `EPD_CLK_HALF_PERIOD_US` (default 1 us, ~500 kHz effective) — 5x the
  proven MicroPython rate, half the known-bad threshold. Framebuffer send
  drops from ~15-20 s (MicroPython) to ~3-5 s; the panel's own refresh
  stays ~20-30 s.
- Panel safety rules (from DRIVER_REFERENCE.md, all enforced in code):
  - `epd_sleep()` after every refresh, on all paths including errors.
  - >= 3 minutes between refreshes.
  - >= 1 refresh per 24 h while powered.
  - One `epd_display()` per composed image; never partial sends.

## 3. Architecture

Single binary, single-threaded superloop, `pico_cyw43_arch_lwip_poll`
(polling mode — no IRQ/threading complexity). No dynamic allocation in
project code; all buffers static. mbedTLS allocates internally from the SDK
heap during handshakes (~50 KB peak) and frees it after.

Boot sequence:

1. `stdio_usb` init (USB serial logging), hardware watchdog armed (~8 s,
   fed inside every wait loop, including the ~35 s refresh).
2. WiFi connect, retry with exponential backoff (5 s doubling to 80 s cap).
3. SNTP time sync (required for TLS certificate time validation, the
   banner date, overdue-deadline coloring, and the "updated at" footer).
4. Odoo `authenticate` once; cache `uid`. Re-authenticate automatically if
   a later call fails with an auth error.
5. Main loop.

Main loop (every `POLL_INTERVAL_S`, default 300):

```
fetch tasks via HTTPS JSON-RPC -> snapshot (fixed struct array)
changed   = memcmp(snapshot, displayed_snapshot) != 0
         || offline_flag != displayed_offline_flag
stale_24h = now - last_refresh >= 24 h
spaced    = now - last_refresh >= 3 min
if (changed || stale_24h) && spaced:
    render framebuffer; epd_display(); epd_sleep()
    displayed_snapshot = snapshot; last_refresh = now
else:
    nothing — panel keeps its image, zero wear
```

No boot splash: the first refresh happens when the first data arrives.
Boot feedback = onboard LED + serial log.

## 4. Repository layout & modules

```
├── CMakeLists.txt
├── pico_sdk_import.cmake
├── lwipopts.h                  lwIP tuning (based on pico-examples tls_client)
├── mbedtls_config.h            mbedTLS tuning (same source)
├── .gitignore                  include/config.h, build/, .remember/
├── config.h.example            committed template -> copy to include/config.h
├── README.md                   zero-to-flashed walkthrough
├── docs/superpowers/specs/     this document
├── lib/jsmn/jsmn.h             vendored single-header JSON tokenizer (MIT)
├── micropython/                previous main.py / dashboard.py, kept as the
│                               validated reference implementation
├── tests/                      host-side tests (see §12)
├── include/                    one header per module, include guards
│   ├── config.h                (gitignored, user-created)
│   ├── epd.h  gfx.h  font8x8.h  net_wifi.h  net_time.h
│   ├── http_client.h  odoo.h  dashboard.h
└── src/
    ├── main.c          boot sequence + superloop + refresh decision
    ├── epd_io.c        paced bit-bang: reset, tx byte, command, data, busy-wait
    ├── epd_driver.c    init sequence, power-on check (3 attempts),
    │                   display, clear, sleep — direct port of main.py
    ├── gfx.c           4bpp framebuffer: pixel, fill, fill_rect, rect
    ├── gfx_text.c      scaled 8x8 text, centered text (font8x8 public domain)
    ├── net_wifi.c      connect / reconnect / link status
    ├── net_time.c      SNTP sync, epoch->local helpers
    ├── http_client.c   one-shot HTTPS POST over lwIP altcp_tls
    ├── odoo_client.c   JSON-RPC request building: authenticate, search_read
    ├── odoo_parse.c    jsmn walk -> struct odoo_task[], UTF-8->ASCII folding
    └── dashboard.c     layout/render of the task board
```

The split keeps every file within C-O3 limits. `odoo_parse.c` and the
refresh-decision helper are written without any Pico SDK dependency so they
compile and test on the host PC.

Data model (all static):

```c
struct odoo_task {
    char name[64];
    char project[32];
    char deadline[11];      /* "YYYY-MM-DD" or "" */
    char stage[24];
    unsigned int priority;  /* 0 normal, 1 high */
};
/* MAX_TASKS = 12 (rows that fit legibly on 800x480) */
```

Fields longer than the buffer are truncated with a trailing `.` ellipsis.

## 5. Network: WiFi, SNTP, TLS

- `pico_cyw43_arch_lwip_poll`; `cyw43_arch_poll()` called in every wait
  loop. Country code from config (`WIFI_COUNTRY`).
- SNTP via lwIP's SNTP app; pool server configurable
  (`NTP_SERVER`, default "pool.ntp.org"); timezone offset in config
  (`TZ_OFFSET_MIN`) — no DST database, a plain minute offset (YAGNI).
- TLS via lwIP `altcp_tls` + mbedTLS. `ODOO_CA_CERT` in config:
  - PEM string set -> full certificate verification against that CA
    (config.h.example documents the `openssl s_client -showcerts` one-liner
    to fetch it).
  - `NULL` -> TLS encrypts but does not verify (documented trade-off: a
    LAN MITM could capture the API key).

## 6. Odoo protocol

JSON-RPC external API, `POST /jsonrpc`, two operations:

1. `common.authenticate(db, login, api_key, {})` -> `uid` (int). Called at
   boot; re-called once automatically when a poll returns an Odoo auth
   error, then the poll is retried.
2. `object.execute_kw(db, uid, api_key, "project.task", "search_read",
   [domain], {"fields": ["name","project_id","date_deadline","stage_id",
   "priority"], "limit": 12, "order": "date_deadline asc, priority desc"})`.

The domain is a printf template in config so it can be adapted to any Odoo
version or filter without touching code (`%d` is replaced by `uid`):

```c
#define ODOO_TASK_DOMAIN "[[\"user_ids\",\"in\",[%d]],[\"is_closed\",\"=\",false]]"
```

If the instance's version lacks `is_closed`, the user edits this one line
(e.g. filter on `stage_id.fold` or drop the clause). The README documents
this.

Responses: `project_id` and `stage_id` arrive as `[id, "Name"]` pairs; the
display keeps the names. `result` may also be `{"error": {...}}` — parsed
and logged with the Odoo error message.

## 7. HTTP client

Hand-rolled one-shot client (~150 lines), on purpose:

- `HTTP/1.0` + `Connection: close` — an HTTP/1.0 server response may not
  use chunked transfer encoding, so the body is simply "everything until
  the connection closes". No chunked decoder, no streaming parser.
- One fresh TLS connection per poll (a 5-minute-idle persistent connection
  would be dropped by any server anyway).
- Static buffers: 4 KB request, 16 KB response. A response that overflows
  16 KB is treated as an error (with `limit: 12` and 5 small fields, real
  responses are ~2-4 KB).
- Parses only the status line (expects 200) and skips headers to the blank
  line; the body is handed to `odoo_parse`.
- Timeouts: DNS 10 s, TCP/TLS connect 15 s, full response 30 s.

## 8. Display rendering

Graphics primitives (`gfx.c`/`gfx_text.c`): pixel, fill, fill_rect, rect,
text at integer scale (pixel duplication of the public-domain 8x8 font,
same technique as MicroPython `big_text`), centered text. ASCII only; a
UTF-8 folding pass in `odoo_parse.c` maps French accents to ASCII
(é/è/ê/ë->e, à/â->a, ç->c, ô->o, û/ù->u, î/ï->i, œ->oe, uppercase
equivalents likewise; any other multi-byte sequence -> '?').

Layout (800x480):

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

- Row: RED `*` if priority high; task name scale 2 (truncated); project
  name; deadline `DD/MM` — RED when overdue, BLUE when due today or
  tomorrow, BLACK otherwise; stage name. Thin separator line between rows.
- Banner date changes once per day, which naturally coincides with the
  24 h health refresh.
- Footer time = timestamp of the last *screen update* (not a live clock —
  a ticking clock would force a refresh every minute).
- Overflow: `+N autres` in the footer when more than 12 tasks match.
- Empty state: `Aucune tache ouverte` centered, GREEN.
- Offline state (see §9): footer shows `HORS LIGNE depuis HH:MM` in RED.

## 9. Refresh & failure policy

- Refresh iff (snapshot changed by memcmp OR 24 h elapsed) AND >= 3 min
  since previous refresh. Always `epd_sleep()` afterwards.
- A single failed poll: keep the current image, log, retry next cycle.
  No panel activity.
- 3 consecutive failed polls: that *is* a state change -> one refresh with
  the offline footer. Recovery after >= 1 successful poll -> normal render
  (footer change counts as data change).
- WiFi loss: `net_wifi` reconnects with backoff independently; polls
  simply fail (and count) until the link returns.
- Panel unresponsive at boot (BUSY never reacts, 3 attempts): LED blink
  pattern + serial message, watchdog reboot after 30 s (covers transient
  contact issues; persistent wiring faults keep the documented diagnostic
  from DRIVER_REFERENCE.md §6).
- Any hang anywhere: hardware watchdog reboot.

## 10. Configuration & secrets

There are no environment variables on bare metal; the embedded equivalent
is a compile-time config header, gitignored. `config.h.example` (committed):

```c
#define WIFI_SSID              "..."
#define WIFI_PASSWORD          "..."
#define WIFI_COUNTRY           CYW43_COUNTRY_FRANCE
#define ODOO_HOST              "odoo.example.com"
#define ODOO_PORT              443
#define ODOO_DB                "mydb"
#define ODOO_LOGIN             "me@example.com"
#define ODOO_API_KEY           "..."
#define ODOO_CA_CERT           NULL          /* or PEM string */
#define ODOO_TASK_DOMAIN       "[[\"user_ids\",\"in\",[%d]],[\"is_closed\",\"=\",false]]"
#define POLL_INTERVAL_S        300
#define NTP_SERVER             "pool.ntp.org"
#define TZ_OFFSET_MIN          60
#define EPD_CLK_HALF_PERIOD_US 1
```

`.gitignore` contains `include/config.h` from the very first commit, so
credentials can never be committed. The build fails with a clear
`#error` message if `config.h` is missing.

## 11. Memory budget (520 KB SRAM)

| Item                          | ~Size    |
|-------------------------------|----------|
| Framebuffer (static)          | 187.5 KB |
| lwIP pools + CYW43 driver     | ~35 KB   |
| mbedTLS handshake peak (heap) | ~50 KB   |
| HTTP request + response bufs  | 20 KB    |
| Task snapshots (2x12 structs) | ~3 KB    |
| Stack, misc                   | ~20 KB   |
| **Total peak**                | **~315 KB** |

Comfortable margin; no OOM risk path.

## 12. Testing

- Host-side (`tests/`, plain C, any host compiler, no Pico deps):
  - `odoo_parse` against canned responses: nominal 12 tasks, empty list,
    Odoo error object, truncated JSON, oversized fields, accented text.
  - Refresh-decision function: changed/unchanged/24 h/3 min-gap matrix.
- On-hardware validation checklist (serial log staged):
  1. WiFi connected (IP logged), 2. SNTP time set, 3. auth ok (uid
  logged), 4. N tasks fetched, 5. refresh performed / skipped decision
  logged, 6. panel sleeps after refresh.
  Then: router-unplug reconnect test; offline-footer appears after 3
  failures; `git status` shows `include/config.h` untracked.

## 13. Build & toolchain (from zero, Windows 11)

1. Install VS Code + official "Raspberry Pi Pico" extension (bundles
   Pico SDK 2.x, ARM GCC, CMake, Ninja, picotool).
2. Open this folder; the extension configures the CMake project
   (board `pico2_w`).
3. Copy `config.h.example` -> `include/config.h`, fill credentials.
4. Build -> `build/epaper_dashboard.uf2`.
5. Hold BOOTSEL, plug the Pico, drop the .uf2 on the `RP2350` drive.
6. Open the serial monitor (115200) to watch the staged boot log.

CMake links: `pico_stdlib`, `pico_cyw43_arch_lwip_poll`,
`pico_lwip_mbedtls`, `pico_mbedtls`, `hardware_watchdog`.

## 14. Out of scope (YAGNI)

- Partial panel refresh (hardware cannot do it), dithering/images/photos,
  touch/buttons, more than one Odoo query, OTA updates, FreeRTOS,
  deep-sleep power optimization (device is mains/USB powered), non-ASCII
  font rendering, filesystem/runtime configuration.
