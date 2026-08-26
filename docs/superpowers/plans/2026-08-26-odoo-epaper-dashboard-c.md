# Odoo e-Paper Dashboard (C firmware) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Bare-metal C firmware for a Raspberry Pi Pico 2 W that polls a self-hosted Odoo over HTTPS every 5 minutes and shows the user's open tasks on a Waveshare 7.3" Spectra-6 e-paper panel, refreshing only when data changes.

**Architecture:** Single-threaded superloop on the Pico SDK (`pico_cyw43_arch_lwip_poll`), lwIP `altcp_tls` + mbedTLS for HTTPS, hand-rolled HTTP/1.0 POST client, JSON-RPC to Odoo, jsmn-based parsing, custom 4bpp framebuffer graphics, and a C port of the validated MicroPython bit-bang panel driver (paced to ~500 kHz).

**Tech Stack:** Pico SDK 2.x (board `pico2_w`), CMake + Ninja + arm-none-eabi-gcc, lwIP, mbedTLS, jsmn (vendored), font8x8 (vendored), host tests via w64devkit gcc + make.

**Spec:** `docs/superpowers/specs/2026-08-26-odoo-epaper-dashboard-c-design.md`

## Global Constraints

- **Epitech C Coding Style v7 applies to every `.c`/`.h` in `src/`, `include/`, `tests/` and to `tests/Makefile`.** Vendored code in `lib/` is exempt. Key rules (violations are task failures):
  - snake_case everywhere; typedefs end `_t`; macros/global constants/enum members UPPER_SNAKE_CASE.
  - ≤ 80 columns; 4-space indent, never tabs (C files); LF line endings; no trailing spaces; file ends with exactly one newline.
  - Function bodies ≤ 20 lines; ≤ 4 parameters; per `.c` file ≤ 5 exported and ≤ 10 total functions; functions with no params take `void`.
  - Variables declared at top of function, one per statement, one blank line after declarations, no other blank lines inside functions; no comments inside function bodies.
  - Structs passed by pointer (C-F7); pointer `*` sticks to the name (C-V3); `const` wherever the data isn't modified (C-A1).
  - `if/else if/else` ≤ 3 branches; nesting depth ≤ 2 (an `else if` counts as 2); no `goto`; ternaries only simple and value-used.
  - Structure/enum declarations, prototypes, macros live in headers only (C-H1); include guards `#ifndef NAME_H_` (never `#pragma once`); preprocessor directives indented 4 spaces per level of *directive* nesting, independent of C code (C-G3).
  - Global variables forbidden; use `static` locals inside functions for persistent buffers; file-local helpers are `static` (C-A4). `static const` lookup tables inside `.c` files are accepted.
  - One statement per line (C-L1); exactly one empty line between function definitions (C-G2).
- **Every** `.c`/`.h` file starts with this exact header (adapt the description line):

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** <one-line description of the file>
*/
```

  `tests/Makefile` uses the `##` comment variant of the same header.
- **No dynamic allocation in project code.** Large/persistent buffers are `static` locals. mbedTLS allocates internally — that's allowed.
- **Build gates:** host-testable modules must pass `make -C tests run` (gcc, `-Wall -Wextra -Werror`); firmware tasks must produce `build/epaper_dashboard.uf2` via `cmake --build build`. Firmware compiles with `-Wall -Wextra` (no `-Werror` — SDK sources compile inside our target).
- **Hardware checkpoints** are marked `[HW]`: build the .uf2, then ask the user to flash (BOOTSEL drag-drop) and paste serial output. Do NOT block later software tasks on `[HW]` confirmation — continue and batch hardware validation at Task 19.
- **Environment facts** (verified 2026-08-26): Windows 11; `winget` and `python` (3.13) available; `cmake`, `ninja`, `arm-none-eabi-gcc`, `gcc`, `make` all missing; no pico-sdk anywhere. `winget`/env-var commands run in PowerShell; git/curl/build commands work in Bash. After `[Environment]::SetEnvironmentVariable(...,'User')`, also set the variable in the current session (`$env:X = ...`) — new values don't apply retroactively.
- **Documented deviations from the spec** (approved rationale, reconcile in spec later if desired):
  1. TLS certificate *time-validity* is not checked (`MBEDTLS_HAVE_TIME` off — avoids newlib clock syscall plumbing); CA chain + SNI hostname are still verified when `ODOO_CA_CERT` is set. SNTP remains required for dates/clock display.
  2. Footer overflow indicator is the generic `+ d'autres taches` (an exact `+N` would need a second Odoo query, out of scope).
  3. `ODOO_CA_CERT` is *defined-or-absent* in `config.h` (checked with `#ifdef`) instead of a `NULL` sentinel — avoids `-Waddress` warnings.
  4. Extra files beyond the spec's list, forced by the ≤10-functions rule: `jsmn_util.c`, `http_conn.c`, `sys_idle.c`, `include/app.h`.

---

### Task 1: Toolchain bootstrap

**Files:** none in repo (installs tools, clones SDK, sets env vars).

**Interfaces:**
- Consumes: nothing.
- Produces: working `cmake`, `ninja`, `arm-none-eabi-gcc`, host `gcc` + `make`; `C:\Users\charl\pico-sdk` with submodules; user env vars `PICO_SDK_PATH`, `PICO_TOOLCHAIN_PATH`.

- [ ] **Step 1: Install the four toolchain packages (PowerShell)**

```powershell
winget install -e --id Kitware.CMake --accept-source-agreements --accept-package-agreements
winget install -e --id Ninja-build.Ninja --accept-package-agreements
winget install -e --id Arm.GnuArmEmbeddedToolchain --accept-package-agreements
winget install -e --id skeeto.w64devkit --accept-package-agreements
```

If a winget ID is not found, search (`winget search cmake`, `winget search "arm gnu"`, `winget search w64devkit`) and install the matching package. Any working host gcc+make (e.g. MSYS2) is an acceptable substitute for w64devkit.

- [ ] **Step 2: Wire up PATH / env vars (PowerShell)**

Ninja lands in `%LOCALAPPDATA%\Microsoft\WinGet\Links` (already on PATH). CMake installs to `C:\Program Files\CMake\bin` — add to user PATH if `cmake --version` fails. Then:

```powershell
$armRoot = 'C:\Program Files (x86)\Arm GNU Toolchain arm-none-eabi'
$arm = (Get-ChildItem $armRoot | Sort-Object Name | Select-Object -Last 1).FullName + '\bin'
[Environment]::SetEnvironmentVariable('PICO_TOOLCHAIN_PATH', $arm, 'User')
$w64 = (Get-ChildItem "$env:LOCALAPPDATA\Microsoft\WinGet\Packages" -Directory -Filter '*w64devkit*' | Select-Object -First 1).FullName
$w64bin = (Get-ChildItem $w64 -Recurse -Directory -Filter bin | Select-Object -First 1).FullName
[Environment]::SetEnvironmentVariable('Path', [Environment]::GetEnvironmentVariable('Path','User') + ';' + $w64bin, 'User')
```

Adapt discovery paths to what the installers actually did (`Get-ChildItem` first). Also set `$env:Path`/`$env:PICO_TOOLCHAIN_PATH` in-session before verifying.

- [ ] **Step 3: Clone the Pico SDK (Bash)**

```bash
git ls-remote --tags https://github.com/raspberrypi/pico-sdk.git | grep -o 'refs/tags/2\.[0-9.]*$' | sort -V | tail -3
# pick the highest stable 2.x.y tag printed, e.g. 2.1.1:
git clone --depth 1 --branch <TAG> https://github.com/raspberrypi/pico-sdk.git /c/Users/charl/pico-sdk
cd /c/Users/charl/pico-sdk
git submodule update --init --depth 1 lib/cyw43-driver lib/lwip lib/mbedtls lib/tinyusb
```

```powershell
[Environment]::SetEnvironmentVariable('PICO_SDK_PATH', 'C:\Users\charl\pico-sdk', 'User')
```

- [ ] **Step 4: Verify everything**

Run (fresh env in-session): `cmake --version`, `ninja --version`, `arm-none-eabi-gcc --version` (use `$env:PICO_TOOLCHAIN_PATH\arm-none-eabi-gcc.exe` if not on PATH — that's fine, the SDK uses the env var), `gcc --version`, `make --version`, `ls $env:PICO_SDK_PATH/pico_sdk_init.cmake`.
Expected: all print versions / the file exists. Nothing to commit.

---

### Task 2: Repository scaffold

**Files:**
- Create: `.gitattributes`, `config.h.example`, `micropython/` (moved files)
- Modify: none.

**Interfaces:**
- Consumes: nothing.
- Produces: `config.h.example` — the exact macro set every later module's `#include "config.h"` relies on.

- [ ] **Step 1: Move MicroPython reference files (Bash)**

```bash
cd /c/Users/charl/Documents/GitHub/e-paper_pico-2W
mkdir -p micropython src include lib tests
git mv main.py dashboard.py tutorial.md micropython/
```

(`DRIVER_REFERENCE.md` stays at repo root — it documents hardware truth the C driver uses.)

- [ ] **Step 2: Create `.gitattributes`**

```
* text=auto
*.c text eol=lf
*.h text eol=lf
*.md text eol=lf
*.py text eol=lf
Makefile text eol=lf
CMakeLists.txt text eol=lf
*.cmake text eol=lf
```

- [ ] **Step 3: Create `config.h.example`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Configuration template: copy to include/config.h and fill in real values
*/

#ifndef CONFIG_H_
    #define CONFIG_H_

#define WIFI_SSID "MyNetwork"
#define WIFI_PASSWORD "MyPassword"
#define WIFI_COUNTRY CYW43_COUNTRY_FRANCE
#define ODOO_HOST "odoo.example.com"
#define ODOO_PORT 443
#define ODOO_DB "mydb"
#define ODOO_LOGIN "me@example.com"
#define ODOO_API_KEY "paste-api-key-here"
#define ODOO_TASK_DOMAIN "[[\"user_ids\",\"in\",[%d]],[\"is_closed\",\"=\",false]]"
#define POLL_INTERVAL_S 300
#define NTP_SERVER "pool.ntp.org"
#define TZ_OFFSET_MIN 120
#define EPD_CLK_HALF_PERIOD_US 1

/*
** Optional: TLS certificate pinning. When ODOO_CA_CERT is NOT defined the
** connection is encrypted but the server certificate is NOT verified.
** To pin your CA, define it as one C string with \n separators, e.g.:
** #define ODOO_CA_CERT "-----BEGIN CERTIFICATE-----\nMIIB...\n" \
**     "...\n-----END CERTIFICATE-----\n"
** (Your personal include/config.h is gitignored and exempt from the
** single-line-macro style rule.)
*/

#endif /* !CONFIG_H_ */
```

- [ ] **Step 4: Commit**

```bash
git add .gitattributes config.h.example
git commit -m "chore: scaffold repo layout, line-ending policy and config template"
```

(Include the standard trailer block from Global Constraints in this and every commit.)

---

### Task 3: Firmware skeleton builds a .uf2

**Files:**
- Create: `CMakeLists.txt`, `pico_sdk_import.cmake` (copied from SDK), `src/main.c` (stub, replaced in Tasks 15/19).

**Interfaces:**
- Consumes: Task 1 toolchain + `PICO_SDK_PATH`.
- Produces: `build/epaper_dashboard.uf2`; the CMake target `epaper_dashboard` that later tasks extend.

- [ ] **Step 1: Copy the SDK import shim (Bash)**

```bash
cp "$PICO_SDK_PATH/external/pico_sdk_import.cmake" pico_sdk_import.cmake
```

- [ ] **Step 2: Create `CMakeLists.txt`**

```cmake
cmake_minimum_required(VERSION 3.13)

set(PICO_BOARD pico2_w CACHE STRING "Board type")
include(pico_sdk_import.cmake)

project(epaper_dashboard C CXX ASM)
set(CMAKE_C_STANDARD 11)

pico_sdk_init()

add_executable(epaper_dashboard
    src/main.c
)

target_include_directories(epaper_dashboard PRIVATE
    ${CMAKE_CURRENT_LIST_DIR}
    ${CMAKE_CURRENT_LIST_DIR}/include
    ${CMAKE_CURRENT_LIST_DIR}/lib/jsmn
    ${CMAKE_CURRENT_LIST_DIR}/lib/font8x8
)

target_compile_options(epaper_dashboard PRIVATE -Wall -Wextra)

target_link_libraries(epaper_dashboard
    pico_stdlib
)

pico_enable_stdio_usb(epaper_dashboard 1)
pico_enable_stdio_uart(epaper_dashboard 0)
pico_add_extra_outputs(epaper_dashboard)
```

- [ ] **Step 3: Create stub `src/main.c`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Entry point (toolchain-check stub, replaced by the real superloop later)
*/

#include <stdio.h>
#include "pico/stdlib.h"

int main(void)
{
    stdio_init_all();
    for (;;) {
        printf("epaper_dashboard: toolchain OK\n");
        sleep_ms(1000);
    }
    return 0;
}
```

- [ ] **Step 4: Build**

```bash
cmake -S . -B build -G Ninja
cmake --build build
ls build/epaper_dashboard.uf2
```

Expected: `.uf2` exists. First configure downloads picotool — allow it.

- [ ] **Step 5 [HW]:** Offer the user to flash this stub (BOOTSEL drag-drop, serial monitor 115200 should print `toolchain OK`). Do not block on it.

- [ ] **Step 6: Commit**

```bash
git add CMakeLists.txt pico_sdk_import.cmake src/main.c
git commit -m "feat: minimal Pico 2 W firmware skeleton builds a uf2"
```

---

### Task 4: Vendored libraries + host test harness

**Files:**
- Create: `lib/jsmn/jsmn.h`, `lib/font8x8/font8x8_basic.h`, `tests/Makefile`, `tests/test_config/config.h`, `tests/test_harness.c`.

**Interfaces:**
- Consumes: host gcc + make (Task 1).
- Produces: `make -C tests run` convention; `tests/test_config/config.h` fake credentials all request-builder tests compile against; jsmn (`jsmntok_t`, `jsmn_parse`) and `font8x8_basic[128][8]` for later tasks.

- [ ] **Step 1: Vendor the two single-header libraries (Bash)**

```bash
curl -fsSL https://raw.githubusercontent.com/zserge/jsmn/master/jsmn.h -o lib/jsmn/jsmn.h
curl -fsSL https://raw.githubusercontent.com/dhepper/font8x8/master/font8x8_basic.h -o lib/font8x8/font8x8_basic.h
grep -q jsmn_parse lib/jsmn/jsmn.h && grep -q font8x8_basic lib/font8x8/font8x8_basic.h && echo VENDOR_OK
```

Expected: `VENDOR_OK`. (jsmn: MIT license; font8x8: public domain — keep their headers intact. `font8x8_basic.h` defines a non-static array: it must be included by exactly one `.c` file, `src/gfx_text.c`.)

- [ ] **Step 2: Create `tests/test_config/config.h`** (fake credentials, same shape as `config.h.example`)

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Fake configuration used by host-side unit tests only
*/

#ifndef CONFIG_H_
    #define CONFIG_H_

#define WIFI_SSID "test-ssid"
#define WIFI_PASSWORD "test-pass"
#define ODOO_HOST "odoo.test.lan"
#define ODOO_PORT 443
#define ODOO_DB "testdb"
#define ODOO_LOGIN "tester@test.lan"
#define ODOO_API_KEY "test-key"
#define ODOO_TASK_DOMAIN "[[\"user_ids\",\"in\",[%d]]]"
#define POLL_INTERVAL_S 300
#define NTP_SERVER "pool.ntp.org"
#define TZ_OFFSET_MIN 120
#define EPD_CLK_HALF_PERIOD_US 1

#endif /* !CONFIG_H_ */
```

- [ ] **Step 3: Create `tests/test_harness.c`** (proves the harness itself works)

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Smoke test proving the host test harness compiles and runs
*/

#include <assert.h>
#include <stdio.h>
#include "config.h"

int main(void)
{
    assert(ODOO_PORT == 443);
    printf("test_harness: OK\n");
    return 0;
}
```

- [ ] **Step 4: Create `tests/Makefile`**

```make
##
## EPITECH PROJECT, 2026
## epaper_dashboard
## File description:
## Host-side unit test build and runner (gcc + make from w64devkit)
##

CC = gcc
CFLAGS = -Wall -Wextra -Werror -std=gnu11 -Itest_config -I../include \
	-I../lib/jsmn -I../lib/font8x8

TESTS = test_harness

all: $(addsuffix .exe,$(TESTS))

test_harness.exe: test_harness.c
	$(CC) $(CFLAGS) -o $@ $^

run: all
	for t in $(TESTS); do ./$$t.exe || exit 1; done

clean:
	rm -f *.exe
```

(Recipe lines use real tabs — mandatory in make. Each later task appends its own rule and extends `TESTS`.)

- [ ] **Step 5: Run** `make -C tests run` — Expected: `test_harness: OK`.

- [ ] **Step 6: Commit**

```bash
git add lib tests
git commit -m "feat: vendor jsmn and font8x8, add host test harness"
```

---

### Task 5: `json_str` — JSON string extraction with ASCII folding

**Files:**
- Create: `include/json_str.h`, `src/json_str.c`, `tests/test_json_str.c`
- Modify: `tests/Makefile`

**Interfaces:**
- Consumes: nothing (pure C, no Pico deps).
- Produces: `void json_str_fold(char *dst, size_t size, char const *src, size_t len);` — decodes a raw JSON string token (`src`/`len` = bytes between the quotes, escapes intact), folds UTF-8 and `\uXXXX` to ASCII (é→e, œ→oe, …), always NUL-terminates, and marks truncation by making the last kept char `'.'`.

- [ ] **Step 1: Write the failing test `tests/test_json_str.c`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Unit tests for JSON string extraction and ASCII folding
*/

#include <assert.h>
#include <stdio.h>
#include <string.h>
#include "json_str.h"

static void test_plain_ascii(void)
{
    char out[16];

    json_str_fold(out, sizeof(out), "hello", 5);
    assert(strcmp(out, "hello") == 0);
}

static void test_unicode_escape(void)
{
    char out[16];

    json_str_fold(out, sizeof(out), "caf\\u00e9", 9);
    assert(strcmp(out, "cafe") == 0);
}

static void test_raw_utf8(void)
{
    char out[16];

    json_str_fold(out, sizeof(out), "d\xc3\xa9j\xc3\xa0 vu", 9);
    assert(strcmp(out, "deja vu") == 0);
}

static void test_escapes(void)
{
    char out[16];

    json_str_fold(out, sizeof(out), "a\\\"b\\\\c\\nd", 10);
    assert(strcmp(out, "a\"b\\c d") == 0);
}

static void test_truncation(void)
{
    char out[6];

    json_str_fold(out, sizeof(out), "abcdefgh", 8);
    assert(strcmp(out, "abcd.") == 0);
}

static void test_oe_ligature(void)
{
    char out[16];

    json_str_fold(out, sizeof(out), "c\xc5\x93ur", 5);
    assert(strcmp(out, "coeur") == 0);
}

int main(void)
{
    test_plain_ascii();
    test_unicode_escape();
    test_raw_utf8();
    test_escapes();
    test_truncation();
    test_oe_ligature();
    printf("test_json_str: OK\n");
    return 0;
}
```

- [ ] **Step 2: Extend `tests/Makefile`** — set `TESTS = test_harness test_json_str` and append:

```make
test_json_str.exe: test_json_str.c ../src/json_str.c
	$(CC) $(CFLAGS) -o $@ $^
```

Run `make -C tests run`. Expected: FAIL (no such file `../src/json_str.c` / missing header).

- [ ] **Step 3: Create `include/json_str.h`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** JSON string token extraction with UTF-8 and escape folding to ASCII
*/

#ifndef JSON_STR_H_
    #define JSON_STR_H_

    #include <stddef.h>
    #include <stdint.h>

struct json_out {
    char *dst;
    size_t size;
    size_t pos;
};

struct fold_entry {
    uint32_t cp;
    char const *out;
};

void json_str_fold(char *dst, size_t size, char const *src, size_t len);

#endif /* !JSON_STR_H_ */
```

- [ ] **Step 4: Create `src/json_str.c`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** JSON string token extraction with UTF-8 and escape folding to ASCII
*/

#include "json_str.h"

static const struct fold_entry FOLD_TABLE[] = {
    {0xE0, "a"}, {0xE2, "a"}, {0xE4, "a"}, {0xE7, "c"}, {0xE8, "e"},
    {0xE9, "e"}, {0xEA, "e"}, {0xEB, "e"}, {0xEE, "i"}, {0xEF, "i"},
    {0xF4, "o"}, {0xF6, "o"}, {0xF9, "u"}, {0xFB, "u"}, {0xFC, "u"},
    {0xC0, "A"}, {0xC2, "A"}, {0xC7, "C"}, {0xC8, "E"}, {0xC9, "E"},
    {0xCA, "E"}, {0xCB, "E"}, {0xCE, "I"}, {0xCF, "I"}, {0xD4, "O"},
    {0xD9, "U"}, {0xDB, "U"}, {0x153, "oe"}, {0x152, "OE"},
    {0xE6, "ae"}, {0xC6, "AE"}, {0x2019, "'"}, {0x2013, "-"},
    {0x2014, "-"}, {0x2026, "..."}, {0, 0}
};

static int hex_val(char c)
{
    if (c >= '0' && c <= '9')
        return c - '0';
    if ((c | 32) >= 'a' && (c | 32) <= 'f')
        return (c | 32) - 'a' + 10;
    return -1;
}

static size_t decode_hex4(char const *src, uint32_t *cp)
{
    uint32_t value = 0;
    int digit = 0;

    for (size_t i = 0; i < 4; i++) {
        digit = hex_val(src[i]);
        if (digit < 0)
            return 0;
        value = (value << 4) | (uint32_t)digit;
    }
    *cp = value;
    return 4;
}

static size_t decode_escape(char const *src, size_t len, uint32_t *cp)
{
    *cp = '?';
    if (len < 1)
        return 0;
    if (*src == 'u' && len >= 5 && decode_hex4(src + 1, cp) == 4)
        return 5;
    if (*src == 'u')
        return len < 5 ? len : 5;
    if (*src == 'n' || *src == 'r' || *src == 't')
        *cp = ' ';
    if (*src == '"' || *src == '\\' || *src == '/')
        *cp = (uint32_t)*src;
    return 1;
}

static size_t utf8_len(uint8_t lead)
{
    if (lead < 0x80)
        return 1;
    if ((lead & 0xE0) == 0xC0)
        return 2;
    if ((lead & 0xF0) == 0xE0)
        return 3;
    if ((lead & 0xF8) == 0xF0)
        return 4;
    return 0;
}

static size_t decode_utf8(char const *src, size_t len, uint32_t *cp)
{
    size_t need = utf8_len((uint8_t)*src);
    uint32_t value = 0;

    *cp = '?';
    if ((uint8_t)*src < 0x80) {
        *cp = (uint8_t)*src;
        return 1;
    }
    if (need == 0 || need > len)
        return 1;
    value = (uint8_t)*src & (uint32_t)(0xFF >> (need + 1));
    for (size_t i = 1; i < need; i++) {
        if (((uint8_t)src[i] & 0xC0) != 0x80)
            return 1;
        value = (value << 6) | ((uint8_t)src[i] & 0x3F);
    }
    *cp = value;
    return need;
}

static char const *fold_codepoint(uint32_t cp)
{
    static char single[2];

    if (cp >= 0x20 && cp < 0x7F) {
        single[0] = (char)cp;
        single[1] = '\0';
        return single;
    }
    for (size_t i = 0; FOLD_TABLE[i].out != 0; i++) {
        if (FOLD_TABLE[i].cp == cp)
            return FOLD_TABLE[i].out;
    }
    return cp < 0x20 ? " " : "?";
}

static void append_char(struct json_out *out, char c)
{
    if (out->pos + 1 < out->size) {
        out->dst[out->pos] = c;
        out->pos++;
        return;
    }
    if (out->size > 1)
        out->dst[out->size - 2] = '.';
}

static void append_folded(struct json_out *out, char const *s)
{
    for (size_t i = 0; s[i] != '\0'; i++)
        append_char(out, s[i]);
}

void json_str_fold(char *dst, size_t size, char const *src, size_t len)
{
    struct json_out out = {dst, size, 0};
    size_t i = 0;
    uint32_t cp = 0;

    if (size == 0)
        return;
    while (i < len) {
        if (src[i] == '\\')
            i += 1 + decode_escape(src + i + 1, len - i - 1, &cp);
        else
            i += decode_utf8(src + i, len - i, &cp);
        append_folded(&out, fold_codepoint(cp));
    }
    dst[out.pos] = '\0';
}
```

- [ ] **Step 5: Run** `make -C tests run` — Expected: `test_json_str: OK`.

- [ ] **Step 6: Commit** — `git add include/json_str.h src/json_str.c tests` , message `feat: json string folding to ascii with escape and utf-8 support`.

---

### Task 6: `jsmn_util` + `odoo_parse` — response parsing

**Files:**
- Create: `include/jsmn_util.h`, `src/jsmn_util.c`, `include/odoo.h`, `include/odoo_parse.h`, `src/odoo_parse.c`, `tests/test_odoo_parse.c`
- Modify: `tests/Makefile`

**Interfaces:**
- Consumes: `json_str_fold` (Task 5), vendored `jsmn.h` (Task 4).
- Produces:
  - `struct jsmn_ctx { char const *json; size_t len; jsmntok_t const *toks; int count; };`
  - `int jsmn_util_parse(struct jsmn_ctx *ctx, jsmntok_t *toks, int max);` (caller pre-fills `json`+`len`; fills `toks`/`count`; returns count or -1)
  - `int jsmn_tok_eq(struct jsmn_ctx const *ctx, int idx, char const *s);`
  - `int jsmn_next_sibling(struct jsmn_ctx const *ctx, int idx);`
  - `int jsmn_find_key(struct jsmn_ctx const *ctx, char const *key);` (search root object, returns value index or -1)
  - `include/odoo.h`: `ODOO_MAX_TASKS` (12), `ODOO_FETCH_LIMIT` (13), `ODOO_REQ_CAP` (2048), `struct odoo_task { char name[64]; char project[32]; char deadline[11]; char stage[24]; unsigned int priority; };`, `struct odoo_task_list { unsigned int count; unsigned int overflow; struct odoo_task tasks[ODOO_MAX_TASKS]; };`
  - `int odoo_parse_auth(char const *json, size_t len, int *uid);` (0 ok / -1)
  - `int odoo_parse_tasks(char const *json, size_t len, struct odoo_task_list *list);` (0 ok / -1 parse error / -2 Odoo error object; always memsets `list` first)

- [ ] **Step 1: Write the failing test `tests/test_odoo_parse.c`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Unit tests for Odoo JSON-RPC response parsing
*/

#include <assert.h>
#include <stdio.h>
#include <string.h>
#include "odoo_parse.h"

static const char SAMPLE_AUTH[] =
    "{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":42}";
static const char SAMPLE_AUTH_FAIL[] =
    "{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":false}";
static const char SAMPLE_ERROR[] = "{\"jsonrpc\":\"2.0\",\"id\":2,"
    "\"error\":{\"code\":200,\"message\":\"Odoo Server Error\"}}";
static const char SAMPLE_TASKS[] = "{\"jsonrpc\":\"2.0\",\"id\":2,"
    "\"result\":[{\"id\":7,\"name\":\"R\\u00e9parer le module\","
    "\"project_id\":[3,\"Compta\"],\"date_deadline\":\"2026-08-30\","
    "\"stage_id\":[2,\"En cours\"],\"priority\":\"1\"},"
    "{\"id\":8,\"name\":\"Deuxieme tache\",\"project_id\":false,"
    "\"date_deadline\":false,\"stage_id\":[1,\"A faire\"],"
    "\"priority\":\"0\"}]}";

static void test_auth(void)
{
    int uid = 0;

    assert(odoo_parse_auth(SAMPLE_AUTH, strlen(SAMPLE_AUTH), &uid) == 0);
    assert(uid == 42);
    assert(odoo_parse_auth(SAMPLE_AUTH_FAIL, strlen(SAMPLE_AUTH_FAIL),
        &uid) == -1);
    assert(odoo_parse_auth("garbage", 7, &uid) == -1);
}

static void test_tasks_nominal(void)
{
    struct odoo_task_list list;

    assert(odoo_parse_tasks(SAMPLE_TASKS, strlen(SAMPLE_TASKS),
        &list) == 0);
    assert(list.count == 2);
    assert(list.overflow == 0);
    assert(strcmp(list.tasks[0].name, "Reparer le module") == 0);
    assert(strcmp(list.tasks[0].project, "Compta") == 0);
    assert(strcmp(list.tasks[0].deadline, "2026-08-30") == 0);
    assert(strcmp(list.tasks[0].stage, "En cours") == 0);
    assert(list.tasks[0].priority == 1);
}

static void test_tasks_false_fields(void)
{
    struct odoo_task_list list;

    assert(odoo_parse_tasks(SAMPLE_TASKS, strlen(SAMPLE_TASKS),
        &list) == 0);
    assert(strcmp(list.tasks[1].project, "") == 0);
    assert(strcmp(list.tasks[1].deadline, "") == 0);
    assert(strcmp(list.tasks[1].stage, "A faire") == 0);
    assert(list.tasks[1].priority == 0);
}

static void test_error_and_garbage(void)
{
    struct odoo_task_list list;

    assert(odoo_parse_tasks(SAMPLE_ERROR, strlen(SAMPLE_ERROR),
        &list) == -2);
    assert(odoo_parse_tasks("not json at all", 15, &list) == -1);
}

static void build_many(char *dst, size_t size, int n)
{
    size_t pos = 0;

    pos += (size_t)snprintf(dst, size, "{\"result\":[");
    for (int i = 0; i < n; i++)
        pos += (size_t)snprintf(dst + pos, size - pos,
            "%s{\"id\":%d,\"name\":\"T%d\",\"priority\":\"0\"}",
            i > 0 ? "," : "", i, i);
    snprintf(dst + pos, size - pos, "]}");
}

static void test_overflow(void)
{
    static char json[4096];
    struct odoo_task_list list;

    build_many(json, sizeof(json), 13);
    assert(odoo_parse_tasks(json, strlen(json), &list) == 0);
    assert(list.count == 12);
    assert(list.overflow == 1);
    assert(strcmp(list.tasks[11].name, "T11") == 0);
}

int main(void)
{
    test_auth();
    test_tasks_nominal();
    test_tasks_false_fields();
    test_error_and_garbage();
    test_overflow();
    printf("test_odoo_parse: OK\n");
    return 0;
}
```

- [ ] **Step 2: Extend `tests/Makefile`** — `TESTS = test_harness test_json_str test_odoo_parse`, append:

```make
test_odoo_parse.exe: test_odoo_parse.c ../src/odoo_parse.c \
	../src/jsmn_util.c ../src/json_str.c
	$(CC) $(CFLAGS) -o $@ $^
```

Run: expected FAIL (missing files).

- [ ] **Step 3: Create `include/odoo.h`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Odoo task data model shared by parsing, requests, refresh and rendering
*/

#ifndef ODOO_H_
    #define ODOO_H_

#define ODOO_MAX_TASKS 12
#define ODOO_FETCH_LIMIT (ODOO_MAX_TASKS + 1)
#define ODOO_REQ_CAP 2048

struct odoo_task {
    char name[64];
    char project[32];
    char deadline[11];
    char stage[24];
    unsigned int priority;
};

struct odoo_task_list {
    unsigned int count;
    unsigned int overflow;
    struct odoo_task tasks[ODOO_MAX_TASKS];
};

#endif /* !ODOO_H_ */
```

- [ ] **Step 4: Create `include/jsmn_util.h`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Generic helpers over the jsmn token array
*/

#ifndef JSMN_UTIL_H_
    #define JSMN_UTIL_H_

    #include <stddef.h>
    #include "jsmn.h"

struct jsmn_ctx {
    char const *json;
    size_t len;
    jsmntok_t const *toks;
    int count;
};

int jsmn_util_parse(struct jsmn_ctx *ctx, jsmntok_t *toks, int max);
int jsmn_tok_eq(struct jsmn_ctx const *ctx, int idx, char const *s);
int jsmn_next_sibling(struct jsmn_ctx const *ctx, int idx);
int jsmn_find_key(struct jsmn_ctx const *ctx, char const *key);

#endif /* !JSMN_UTIL_H_ */
```

- [ ] **Step 5: Create `src/jsmn_util.c`** (NOTE: `#define JSMN_STATIC` **before** any include — this pulls the jsmn implementation into this one translation unit)

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Generic helpers over the jsmn token array
*/

#define JSMN_STATIC
#include "jsmn.h"
#include <string.h>
#include "jsmn_util.h"

int jsmn_util_parse(struct jsmn_ctx *ctx, jsmntok_t *toks, int max)
{
    jsmn_parser parser;
    int count = 0;

    jsmn_init(&parser);
    count = jsmn_parse(&parser, ctx->json, ctx->len, toks,
        (unsigned int)max);
    if (count < 1)
        return -1;
    ctx->toks = toks;
    ctx->count = count;
    return count;
}

int jsmn_tok_eq(struct jsmn_ctx const *ctx, int idx, char const *s)
{
    jsmntok_t const *tok = &ctx->toks[idx];
    size_t len = (size_t)(tok->end - tok->start);

    if (tok->type != JSMN_STRING || strlen(s) != len)
        return 0;
    return strncmp(ctx->json + tok->start, s, len) == 0;
}

int jsmn_next_sibling(struct jsmn_ctx const *ctx, int idx)
{
    int end = ctx->toks[idx].end;
    int i = idx + 1;

    while (i < ctx->count && ctx->toks[i].start < end)
        i++;
    return i;
}

int jsmn_find_key(struct jsmn_ctx const *ctx, char const *key)
{
    int i = 1;

    if (ctx->count < 1 || ctx->toks[0].type != JSMN_OBJECT)
        return -1;
    while (i + 1 < ctx->count) {
        if (jsmn_tok_eq(ctx, i, key))
            return i + 1;
        i = jsmn_next_sibling(ctx, i + 1);
    }
    return -1;
}
```

- [ ] **Step 6: Create `include/odoo_parse.h`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Odoo JSON-RPC response parsing into the task data model
*/

#ifndef ODOO_PARSE_H_
    #define ODOO_PARSE_H_

    #include <stddef.h>
    #include "odoo.h"

#define ODOO_MAX_JSON_TOKENS 512

int odoo_parse_auth(char const *json, size_t len, int *uid);
int odoo_parse_tasks(char const *json, size_t len,
    struct odoo_task_list *list);

#endif /* !ODOO_PARSE_H_ */
```

- [ ] **Step 7: Create `src/odoo_parse.c`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Odoo JSON-RPC response parsing into the task data model
*/

#include <stdlib.h>
#include <string.h>
#include "jsmn_util.h"
#include "json_str.h"
#include "odoo_parse.h"

static void copy_tok(struct jsmn_ctx const *ctx, int idx, char *dst,
    size_t size)
{
    jsmntok_t const *tok = &ctx->toks[idx];

    dst[0] = '\0';
    if (idx < ctx->count && tok->type == JSMN_STRING)
        json_str_fold(dst, size, ctx->json + tok->start,
            (size_t)(tok->end - tok->start));
}

static void copy_relation(struct jsmn_ctx const *ctx, int idx, char *dst,
    size_t size)
{
    dst[0] = '\0';
    if (ctx->toks[idx].type == JSMN_ARRAY && ctx->toks[idx].size >= 2)
        copy_tok(ctx, idx + 2, dst, size);
}

static unsigned int priority_of(struct jsmn_ctx const *ctx, int idx)
{
    jsmntok_t const *tok = &ctx->toks[idx];

    if (tok->type == JSMN_STRING || tok->type == JSMN_PRIMITIVE)
        return ctx->json[tok->start] == '1';
    return 0;
}

static void set_task_field(struct jsmn_ctx const *ctx, int key,
    struct odoo_task *task)
{
    int val = key + 1;

    if (jsmn_tok_eq(ctx, key, "name"))
        copy_tok(ctx, val, task->name, sizeof(task->name));
    if (jsmn_tok_eq(ctx, key, "date_deadline"))
        copy_tok(ctx, val, task->deadline, sizeof(task->deadline));
    if (jsmn_tok_eq(ctx, key, "project_id"))
        copy_relation(ctx, val, task->project, sizeof(task->project));
    if (jsmn_tok_eq(ctx, key, "stage_id"))
        copy_relation(ctx, val, task->stage, sizeof(task->stage));
    if (jsmn_tok_eq(ctx, key, "priority"))
        task->priority = priority_of(ctx, val);
}

static void parse_task(struct jsmn_ctx const *ctx, int idx,
    struct odoo_task *task)
{
    int child = idx + 1;
    int pairs = ctx->toks[idx].size;

    for (int n = 0; n < pairs; n++) {
        set_task_field(ctx, child, task);
        child = jsmn_next_sibling(ctx, child + 1);
    }
}

static void parse_task_array(struct jsmn_ctx const *ctx, int idx,
    struct odoo_task_list *list)
{
    int child = idx + 1;
    int total = ctx->toks[idx].size;

    for (int n = 0; n < total; n++) {
        if (n < ODOO_MAX_TASKS)
            parse_task(ctx, child, &list->tasks[n]);
        child = jsmn_next_sibling(ctx, child);
    }
    list->count = total > ODOO_MAX_TASKS ? ODOO_MAX_TASKS
        : (unsigned int)total;
    list->overflow = total > ODOO_MAX_TASKS;
}

int odoo_parse_auth(char const *json, size_t len, int *uid)
{
    static jsmntok_t toks[64];
    struct jsmn_ctx ctx = {json, len, 0, 0};
    int val = 0;

    if (jsmn_util_parse(&ctx, toks, 64) < 0)
        return -1;
    val = jsmn_find_key(&ctx, "result");
    if (val < 0 || val >= ctx.count
        || ctx.toks[val].type != JSMN_PRIMITIVE)
        return -1;
    if (json[ctx.toks[val].start] < '0'
        || json[ctx.toks[val].start] > '9')
        return -1;
    *uid = atoi(json + ctx.toks[val].start);
    return 0;
}

int odoo_parse_tasks(char const *json, size_t len,
    struct odoo_task_list *list)
{
    static jsmntok_t toks[ODOO_MAX_JSON_TOKENS];
    struct jsmn_ctx ctx = {json, len, 0, 0};
    int val = 0;

    memset(list, 0, sizeof(*list));
    if (jsmn_util_parse(&ctx, toks, ODOO_MAX_JSON_TOKENS) < 0)
        return -1;
    if (jsmn_find_key(&ctx, "error") >= 0)
        return -2;
    val = jsmn_find_key(&ctx, "result");
    if (val < 0 || val >= ctx.count || ctx.toks[val].type != JSMN_ARRAY)
        return -1;
    parse_task_array(&ctx, val, list);
    return 0;
}
```

(`odoo_parse_auth` relies on the caller NUL-terminating the buffer for `atoi` — `http_client` guarantees this, and test strings are literals.)

- [ ] **Step 8: Run** `make -C tests run` — Expected: all OK.

- [ ] **Step 9: Commit** — `feat: parse odoo auth and task responses via jsmn`.

---

### Task 7: `odoo_request` — JSON-RPC request builders

**Files:**
- Create: `include/odoo_request.h`, `src/odoo_request.c`, `tests/test_odoo_request.c`
- Modify: `tests/Makefile`

**Interfaces:**
- Consumes: `config.h` macros (`ODOO_DB`, `ODOO_LOGIN`, `ODOO_API_KEY`, `ODOO_TASK_DOMAIN`), `ODOO_FETCH_LIMIT` from `odoo.h`.
- Produces: `int odoo_build_auth(char *dst, size_t size);` and `int odoo_build_tasks(char *dst, size_t size, int uid);` — both return body length, or -1 if truncated.

- [ ] **Step 1: Write the failing test `tests/test_odoo_request.c`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Unit tests for Odoo JSON-RPC request body builders
*/

#include <assert.h>
#include <stdio.h>
#include <string.h>
#include "odoo_request.h"

static void test_auth_body(void)
{
    char body[512];
    int len = odoo_build_auth(body, sizeof(body));

    assert(len > 0);
    assert((size_t)len == strlen(body));
    assert(strstr(body, "\"service\":\"common\"") != 0);
    assert(strstr(body, "\"authenticate\"") != 0);
    assert(strstr(body, "\"testdb\"") != 0);
    assert(strstr(body, "\"tester@test.lan\"") != 0);
    assert(strstr(body, "\"test-key\"") != 0);
}

static void test_tasks_body(void)
{
    char body[2048];
    int len = odoo_build_tasks(body, sizeof(body), 42);

    assert(len > 0);
    assert(strstr(body, "\"execute_kw\"") != 0);
    assert(strstr(body, "\"project.task\"") != 0);
    assert(strstr(body, "\"search_read\"") != 0);
    assert(strstr(body, "[[\"user_ids\",\"in\",[42]]]") != 0);
    assert(strstr(body, "\"limit\":13") != 0);
    assert(strstr(body, "date_deadline asc, priority desc") != 0);
}

static void test_truncation(void)
{
    char body[32];

    assert(odoo_build_auth(body, sizeof(body)) == -1);
    assert(odoo_build_tasks(body, sizeof(body), 1) == -1);
}

int main(void)
{
    test_auth_body();
    test_tasks_body();
    test_truncation();
    printf("test_odoo_request: OK\n");
    return 0;
}
```

- [ ] **Step 2: Extend `tests/Makefile`** — add `test_odoo_request` to `TESTS`, append:

```make
test_odoo_request.exe: test_odoo_request.c ../src/odoo_request.c
	$(CC) $(CFLAGS) -o $@ $^
```

Run: expected FAIL.

- [ ] **Step 3: Create `include/odoo_request.h`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Odoo JSON-RPC request body builders
*/

#ifndef ODOO_REQUEST_H_
    #define ODOO_REQUEST_H_

    #include <stddef.h>

int odoo_build_auth(char *dst, size_t size);
int odoo_build_tasks(char *dst, size_t size, int uid);

#endif /* !ODOO_REQUEST_H_ */
```

- [ ] **Step 4: Create `src/odoo_request.c`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Odoo JSON-RPC request body builders
*/

#include <stdio.h>
#include "config.h"
#include "odoo.h"
#include "odoo_request.h"

static const char AUTH_TEMPLATE[] = "{\"jsonrpc\":\"2.0\","
    "\"method\":\"call\",\"params\":{\"service\":\"common\","
    "\"method\":\"authenticate\",\"args\":[\"%s\",\"%s\",\"%s\",{}]},"
    "\"id\":1}";

static const char TASKS_TEMPLATE[] = "{\"jsonrpc\":\"2.0\","
    "\"method\":\"call\",\"params\":{\"service\":\"object\","
    "\"method\":\"execute_kw\",\"args\":[\"%s\",%d,\"%s\","
    "\"project.task\",\"search_read\",[%s],{\"fields\":[\"name\","
    "\"project_id\",\"date_deadline\",\"stage_id\",\"priority\"],"
    "\"limit\":%d,\"order\":\"date_deadline asc, priority desc\"}]},"
    "\"id\":2}";

int odoo_build_auth(char *dst, size_t size)
{
    int written = snprintf(dst, size, AUTH_TEMPLATE, ODOO_DB, ODOO_LOGIN,
        ODOO_API_KEY);

    if (written < 0 || (size_t)written >= size)
        return -1;
    return written;
}

int odoo_build_tasks(char *dst, size_t size, int uid)
{
    char domain[256];
    int written = snprintf(domain, sizeof(domain), ODOO_TASK_DOMAIN, uid);

    if (written < 0 || (size_t)written >= sizeof(domain))
        return -1;
    written = snprintf(dst, size, TASKS_TEMPLATE, ODOO_DB, uid,
        ODOO_API_KEY, domain, ODOO_FETCH_LIMIT);
    if (written < 0 || (size_t)written >= size)
        return -1;
    return written;
}
```

- [ ] **Step 5: Run** `make -C tests run` — Expected: all OK.
- [ ] **Step 6: Commit** — `feat: build odoo authenticate and search_read request bodies`.

---

### Task 8: `time_fmt` — date/time formatting and deadline classification

**Files:**
- Create: `include/time_fmt.h`, `src/time_fmt.c`, `tests/test_time_fmt.c`
- Modify: `tests/Makefile`

**Interfaces:**
- Consumes: `<time.h>` only.
- Produces:
  - `enum deadline_class { DL_NONE = 0, DL_OVERDUE, DL_SOON, DL_NORMAL };`
  - `void time_fmt_banner(char *dst, size_t size, struct tm const *lt);` → `"mar 26/08"` (French day abbrevs, tm_wday 0=dim)
  - `void time_fmt_hhmm(char *dst, size_t size, struct tm const *lt);` → `"14:35"`
  - `void time_fmt_ddmm(char *dst, size_t size, char const *iso);` → `"30/08"` from `"2026-08-30"`, `"--"` when empty/short
  - `int time_fmt_deadline_class(char const *iso, struct tm const *today);`

- [ ] **Step 1: Write the failing test `tests/test_time_fmt.c`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Unit tests for date formatting and deadline classification
*/

#include <assert.h>
#include <stdio.h>
#include <string.h>
#include <time.h>
#include "time_fmt.h"

static struct tm make_date(int year, int month, int day)
{
    struct tm t = {0};

    t.tm_year = year - 1900;
    t.tm_mon = month - 1;
    t.tm_mday = day;
    t.tm_hour = 12;
    mktime(&t);
    return t;
}

static void test_banner_and_hhmm(void)
{
    struct tm t = make_date(2026, 8, 26);
    char buf[16];

    t.tm_hour = 14;
    t.tm_min = 5;
    time_fmt_banner(buf, sizeof(buf), &t);
    assert(strcmp(buf, "mer 26/08") == 0);
    time_fmt_hhmm(buf, sizeof(buf), &t);
    assert(strcmp(buf, "14:05") == 0);
}

static void test_ddmm(void)
{
    char buf[8];

    time_fmt_ddmm(buf, sizeof(buf), "2026-08-30");
    assert(strcmp(buf, "30/08") == 0);
    time_fmt_ddmm(buf, sizeof(buf), "");
    assert(strcmp(buf, "--") == 0);
}

static void test_deadline_class(void)
{
    struct tm today = make_date(2026, 8, 26);

    assert(time_fmt_deadline_class("2026-08-25", &today) == DL_OVERDUE);
    assert(time_fmt_deadline_class("2026-08-26", &today) == DL_SOON);
    assert(time_fmt_deadline_class("2026-08-27", &today) == DL_SOON);
    assert(time_fmt_deadline_class("2026-08-28", &today) == DL_NORMAL);
    assert(time_fmt_deadline_class("", &today) == DL_NONE);
}

static void test_month_rollover(void)
{
    struct tm today = make_date(2026, 8, 31);

    assert(time_fmt_deadline_class("2026-09-01", &today) == DL_SOON);
    assert(time_fmt_deadline_class("2026-09-02", &today) == DL_NORMAL);
}

int main(void)
{
    test_banner_and_hhmm();
    test_ddmm();
    test_deadline_class();
    test_month_rollover();
    printf("test_time_fmt: OK\n");
    return 0;
}
```

- [ ] **Step 2: Extend `tests/Makefile`** — add `test_time_fmt` to `TESTS`, append:

```make
test_time_fmt.exe: test_time_fmt.c ../src/time_fmt.c
	$(CC) $(CFLAGS) -o $@ $^
```

Run: expected FAIL.

- [ ] **Step 3: Create `include/time_fmt.h`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Date and time formatting helpers plus deadline classification
*/

#ifndef TIME_FMT_H_
    #define TIME_FMT_H_

    #include <stddef.h>
    #include <time.h>

enum deadline_class {
    DL_NONE = 0,
    DL_OVERDUE,
    DL_SOON,
    DL_NORMAL,
};

void time_fmt_banner(char *dst, size_t size, struct tm const *lt);
void time_fmt_hhmm(char *dst, size_t size, struct tm const *lt);
void time_fmt_ddmm(char *dst, size_t size, char const *iso);
int time_fmt_deadline_class(char const *iso, struct tm const *today);

#endif /* !TIME_FMT_H_ */
```

- [ ] **Step 4: Create `src/time_fmt.c`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Date and time formatting helpers plus deadline classification
*/

#include <stdio.h>
#include <string.h>
#include "time_fmt.h"

static const char *DAY_NAMES[7] = {
    "dim", "lun", "mar", "mer", "jeu", "ven", "sam"
};

void time_fmt_banner(char *dst, size_t size, struct tm const *lt)
{
    snprintf(dst, size, "%s %02d/%02d", DAY_NAMES[lt->tm_wday % 7],
        lt->tm_mday, lt->tm_mon + 1);
}

void time_fmt_hhmm(char *dst, size_t size, struct tm const *lt)
{
    snprintf(dst, size, "%02d:%02d", lt->tm_hour, lt->tm_min);
}

void time_fmt_ddmm(char *dst, size_t size, char const *iso)
{
    if (iso == 0 || strlen(iso) < 10) {
        snprintf(dst, size, "--");
        return;
    }
    snprintf(dst, size, "%c%c/%c%c", iso[8], iso[9], iso[5], iso[6]);
}

static void make_iso(char *dst, size_t size, struct tm const *t)
{
    snprintf(dst, size, "%04d-%02d-%02d", t->tm_year + 1900,
        t->tm_mon + 1, t->tm_mday);
}

int time_fmt_deadline_class(char const *iso, struct tm const *today)
{
    char today_iso[11];
    char tomorrow_iso[11];
    struct tm tomorrow = *today;

    if (iso == 0 || iso[0] == '\0')
        return DL_NONE;
    tomorrow.tm_mday++;
    mktime(&tomorrow);
    make_iso(today_iso, sizeof(today_iso), today);
    make_iso(tomorrow_iso, sizeof(tomorrow_iso), &tomorrow);
    if (strncmp(iso, today_iso, 10) < 0)
        return DL_OVERDUE;
    if (strncmp(iso, tomorrow_iso, 10) <= 0)
        return DL_SOON;
    return DL_NORMAL;
}
```

- [ ] **Step 5: Run** `make -C tests run` — Expected: all OK.
- [ ] **Step 6: Commit** — `feat: format banner, clock and deadlines with french day names`.

---

### Task 9: `gfx` — 4bpp framebuffer primitives

**Files:**
- Create: `include/gfx.h`, `src/gfx.c`, `tests/test_gfx.c`
- Modify: `tests/Makefile`

**Interfaces:**
- Consumes: nothing (pure C).
- Produces (also used by `gfx_text`, `dashboard`, `epd_driver`, `main`):
  - `GFX_WIDTH` 800, `GFX_HEIGHT` 480, `GFX_BUFFER_SIZE` (192000)
  - `enum gfx_color { GFX_BLACK = 0x0, GFX_WHITE = 0x1, GFX_YELLOW = 0x2, GFX_RED = 0x3, GFX_BLUE = 0x5, GFX_GREEN = 0x6 };`
  - `struct gfx_rect { int x; int y; int w; int h; };`
  - `struct gfx_style { int x; int y; int color; int scale; };` (consumed by Task 10)
  - `void gfx_fill(uint8_t *fb, int color);` `void gfx_pixel(uint8_t *fb, int x, int y, int color);` `void gfx_fill_rect(uint8_t *fb, struct gfx_rect const *r, int color);` `void gfx_rect(uint8_t *fb, struct gfx_rect const *r, int color);`
  - Pixel packing: GS4_HMSB — byte `(y*800+x)/2`, even x = high nibble. All primitives clip out-of-bounds coordinates silently.

- [ ] **Step 1: Write the failing test `tests/test_gfx.c`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Unit tests for the 4bpp framebuffer primitives
*/

#include <assert.h>
#include <stdio.h>
#include "gfx.h"

static int px(uint8_t const *fb, int x, int y)
{
    size_t idx = ((size_t)y * GFX_WIDTH + (size_t)x) / 2;

    if ((x & 1) == 0)
        return fb[idx] >> 4;
    return fb[idx] & 0x0F;
}

static void test_fill_and_pixel(void)
{
    static uint8_t fb[GFX_BUFFER_SIZE];

    gfx_fill(fb, GFX_WHITE);
    assert(fb[0] == 0x11);
    gfx_pixel(fb, 0, 0, GFX_BLACK);
    assert(fb[0] == 0x01);
    gfx_pixel(fb, 1, 0, GFX_RED);
    assert(fb[0] == 0x03);
    assert(px(fb, 2, 0) == GFX_WHITE);
}

static void test_clipping(void)
{
    static uint8_t fb[GFX_BUFFER_SIZE];
    struct gfx_rect r = {-10, -10, 20, 20};

    gfx_fill(fb, GFX_WHITE);
    gfx_pixel(fb, -1, 0, GFX_BLACK);
    gfx_pixel(fb, GFX_WIDTH, 0, GFX_BLACK);
    gfx_pixel(fb, 0, GFX_HEIGHT, GFX_BLACK);
    gfx_fill_rect(fb, &r, GFX_GREEN);
    assert(px(fb, 0, 0) == GFX_GREEN);
    assert(px(fb, 9, 9) == GFX_GREEN);
    assert(px(fb, 10, 10) == GFX_WHITE);
}

static void test_rect_outline(void)
{
    static uint8_t fb[GFX_BUFFER_SIZE];
    struct gfx_rect r = {10, 10, 5, 5};

    gfx_fill(fb, GFX_WHITE);
    gfx_rect(fb, &r, GFX_BLUE);
    assert(px(fb, 10, 10) == GFX_BLUE);
    assert(px(fb, 14, 14) == GFX_BLUE);
    assert(px(fb, 12, 12) == GFX_WHITE);
}

int main(void)
{
    test_fill_and_pixel();
    test_clipping();
    test_rect_outline();
    printf("test_gfx: OK\n");
    return 0;
}
```

- [ ] **Step 2: Extend `tests/Makefile`** — add `test_gfx` to `TESTS`, append:

```make
test_gfx.exe: test_gfx.c ../src/gfx.c
	$(CC) $(CFLAGS) -o $@ $^
```

Run: expected FAIL.

- [ ] **Step 3: Create `include/gfx.h`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** 4bpp GS4_HMSB framebuffer graphics primitives and text rendering
*/

#ifndef GFX_H_
    #define GFX_H_

    #include <stddef.h>
    #include <stdint.h>

#define GFX_WIDTH 800
#define GFX_HEIGHT 480
#define GFX_BUFFER_SIZE (GFX_WIDTH * GFX_HEIGHT / 2)

enum gfx_color {
    GFX_BLACK = 0x0,
    GFX_WHITE = 0x1,
    GFX_YELLOW = 0x2,
    GFX_RED = 0x3,
    GFX_BLUE = 0x5,
    GFX_GREEN = 0x6,
};

struct gfx_rect {
    int x;
    int y;
    int w;
    int h;
};

struct gfx_style {
    int x;
    int y;
    int color;
    int scale;
};

void gfx_fill(uint8_t *fb, int color);
void gfx_pixel(uint8_t *fb, int x, int y, int color);
void gfx_fill_rect(uint8_t *fb, struct gfx_rect const *r, int color);
void gfx_rect(uint8_t *fb, struct gfx_rect const *r, int color);
void gfx_text(uint8_t *fb, struct gfx_style const *st, char const *s);
void gfx_text_centered(uint8_t *fb, struct gfx_style const *st,
    char const *s);
int gfx_text_width(char const *s, int scale);

#endif /* !GFX_H_ */
```

(The three `gfx_text*` prototypes are implemented in Task 10 — declaring them now keeps one header per logical entity.)

- [ ] **Step 4: Create `src/gfx.c`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** 4bpp GS4_HMSB framebuffer graphics primitives
*/

#include "gfx.h"

void gfx_fill(uint8_t *fb, int color)
{
    uint8_t byte = (uint8_t)((color << 4) | (color & 0x0F));

    for (size_t i = 0; i < GFX_BUFFER_SIZE; i++)
        fb[i] = byte;
}

void gfx_pixel(uint8_t *fb, int x, int y, int color)
{
    size_t idx = 0;

    if (x < 0 || y < 0 || x >= GFX_WIDTH || y >= GFX_HEIGHT)
        return;
    idx = ((size_t)y * GFX_WIDTH + (size_t)x) / 2;
    if ((x & 1) == 0)
        fb[idx] = (uint8_t)((fb[idx] & 0x0F) | (color << 4));
    else
        fb[idx] = (uint8_t)((fb[idx] & 0xF0) | (color & 0x0F));
}

void gfx_fill_rect(uint8_t *fb, struct gfx_rect const *r, int color)
{
    for (int dy = 0; dy < r->h; dy++) {
        for (int dx = 0; dx < r->w; dx++)
            gfx_pixel(fb, r->x + dx, r->y + dy, color);
    }
}

void gfx_rect(uint8_t *fb, struct gfx_rect const *r, int color)
{
    struct gfx_rect top = {r->x, r->y, r->w, 1};
    struct gfx_rect bottom = {r->x, r->y + r->h - 1, r->w, 1};
    struct gfx_rect left = {r->x, r->y, 1, r->h};
    struct gfx_rect right = {r->x + r->w - 1, r->y, 1, r->h};

    gfx_fill_rect(fb, &top, color);
    gfx_fill_rect(fb, &bottom, color);
    gfx_fill_rect(fb, &left, color);
    gfx_fill_rect(fb, &right, color);
}
```

- [ ] **Step 5: Run** `make -C tests run` — Expected: all OK.
- [ ] **Step 6: Commit** — `feat: 4bpp framebuffer fill, pixel and rectangle primitives`.

---

### Task 10: `gfx_text` — scaled 8×8 font rendering

**Files:**
- Create: `src/gfx_text.c`, `tests/test_gfx_text.c`
- Modify: `tests/Makefile`

**Interfaces:**
- Consumes: `gfx.h` (Task 9 — prototypes already declared there), `lib/font8x8/font8x8_basic.h` (Task 4; included by this `.c` ONLY).
- Produces: implementations of `gfx_text`, `gfx_text_centered`, `gfx_text_width`. Glyphs are 8×8, scaled by pixel duplication; advance = `8 * scale` px per char; font8x8 rows are LSB-leftmost (`(bits >> col) & 1`). If text appears mirrored on the real panel, flip to `(bits >> (7 - col)) & 1` — one-line fix, noted here on purpose.

- [ ] **Step 1: Write the failing test `tests/test_gfx_text.c`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Unit tests for scaled bitmap font text rendering
*/

#include <assert.h>
#include <stdio.h>
#include "gfx.h"

static int px(uint8_t const *fb, int x, int y)
{
    size_t idx = ((size_t)y * GFX_WIDTH + (size_t)x) / 2;

    if ((x & 1) == 0)
        return fb[idx] >> 4;
    return fb[idx] & 0x0F;
}

static int count_color(uint8_t const *fb, struct gfx_rect const *r, int c)
{
    int n = 0;

    for (int y = r->y; y < r->y + r->h; y++) {
        for (int x = r->x; x < r->x + r->w; x++)
            n += px(fb, x, y) == c;
    }
    return n;
}

static void test_glyph_draws_pixels(void)
{
    static uint8_t fb[GFX_BUFFER_SIZE];
    struct gfx_style st = {0, 0, GFX_BLACK, 1};
    struct gfx_rect box = {0, 0, 8, 8};

    gfx_fill(fb, GFX_WHITE);
    gfx_text(fb, &st, "A");
    assert(count_color(fb, &box, GFX_BLACK) > 4);
}

static void test_scale_quadruples_area(void)
{
    static uint8_t fb1[GFX_BUFFER_SIZE];
    static uint8_t fb2[GFX_BUFFER_SIZE];
    struct gfx_style st1 = {0, 0, GFX_BLACK, 1};
    struct gfx_style st2 = {0, 0, GFX_BLACK, 2};
    struct gfx_rect box1 = {0, 0, 8, 8};
    struct gfx_rect box2 = {0, 0, 16, 16};

    gfx_fill(fb1, GFX_WHITE);
    gfx_fill(fb2, GFX_WHITE);
    gfx_text(fb1, &st1, "A");
    gfx_text(fb2, &st2, "A");
    assert(count_color(fb2, &box2, GFX_BLACK)
        == 4 * count_color(fb1, &box1, GFX_BLACK));
}

static void test_width_and_centering(void)
{
    static uint8_t fb[GFX_BUFFER_SIZE];
    struct gfx_style st = {0, 100, GFX_RED, 2};
    struct gfx_rect left = {0, 100, 300, 16};

    assert(gfx_text_width("ab", 2) == 32);
    gfx_fill(fb, GFX_WHITE);
    gfx_text_centered(fb, &st, "ab");
    assert(count_color(fb, &left, GFX_RED) == 0);
}

int main(void)
{
    test_glyph_draws_pixels();
    test_scale_quadruples_area();
    test_width_and_centering();
    printf("test_gfx_text: OK\n");
    return 0;
}
```

- [ ] **Step 2: Extend `tests/Makefile`** — add `test_gfx_text` to `TESTS`, append:

```make
test_gfx_text.exe: test_gfx_text.c ../src/gfx_text.c ../src/gfx.c
	$(CC) $(CFLAGS) -o $@ $^
```

Run: expected FAIL.

- [ ] **Step 3: Create `src/gfx_text.c`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Scaled bitmap text rendering using the public-domain font8x8
*/

#include <string.h>
#include "gfx.h"
#include "font8x8_basic.h"

static void draw_dot(uint8_t *fb, struct gfx_style const *st, int col,
    int row)
{
    struct gfx_rect dot = {st->x + col * st->scale,
        st->y + row * st->scale, st->scale, st->scale};

    gfx_fill_rect(fb, &dot, st->color);
}

static void draw_glyph_row(uint8_t *fb, struct gfx_style const *st,
    uint8_t bits, int row)
{
    for (int col = 0; col < 8; col++) {
        if (((bits >> col) & 1) != 0)
            draw_dot(fb, st, col, row);
    }
}

static void draw_glyph(uint8_t *fb, struct gfx_style const *st, char c)
{
    uint8_t const *rows = (uint8_t const *)font8x8_basic[(uint8_t)c & 0x7F];

    for (int row = 0; row < 8; row++)
        draw_glyph_row(fb, st, rows[row], row);
}

void gfx_text(uint8_t *fb, struct gfx_style const *st, char const *s)
{
    struct gfx_style cur = *st;

    for (size_t i = 0; s[i] != '\0'; i++) {
        draw_glyph(fb, &cur, s[i]);
        cur.x += 8 * cur.scale;
    }
}

int gfx_text_width(char const *s, int scale)
{
    return (int)strlen(s) * 8 * scale;
}

void gfx_text_centered(uint8_t *fb, struct gfx_style const *st,
    char const *s)
{
    struct gfx_style cur = *st;

    cur.x = (GFX_WIDTH - gfx_text_width(s, st->scale)) / 2;
    gfx_text(fb, &cur, s);
}
```

- [ ] **Step 4: Run** `make -C tests run` — Expected: all OK.
- [ ] **Step 5: Commit** — `feat: scaled font8x8 text rendering with centering`.

---

### Task 11: `http_util` — request building and response splitting

**Files:**
- Create: `include/http_util.h`, `src/http_util.c`, `tests/test_http_util.c`
- Modify: `tests/Makefile`

**Interfaces:**
- Consumes: `config.h` (`ODOO_HOST`).
- Produces:
  - `int http_build_request(char *dst, size_t size, char const *path, char const *body);` — full HTTP/1.0 POST request (Host, Content-Type json, Content-Length, Connection: close); returns length or -1.
  - `int http_parse_status(char const *resp, size_t len);` — status code or -1.
  - `long http_body_offset(char const *resp, size_t len);` — offset just past `\r\n\r\n` or -1.

- [ ] **Step 1: Write the failing test `tests/test_http_util.c`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Unit tests for HTTP request building and response splitting
*/

#include <assert.h>
#include <stdio.h>
#include <string.h>
#include "http_util.h"

static void test_build_request(void)
{
    char req[512];
    int len = http_build_request(req, sizeof(req), "/jsonrpc", "{}");

    assert(len > 0);
    assert(strncmp(req, "POST /jsonrpc HTTP/1.0\r\n", 24) == 0);
    assert(strstr(req, "Host: odoo.test.lan\r\n") != 0);
    assert(strstr(req, "Content-Type: application/json\r\n") != 0);
    assert(strstr(req, "Content-Length: 2\r\n") != 0);
    assert(strstr(req, "Connection: close\r\n\r\n{}") != 0);
}

static void test_build_truncation(void)
{
    char req[16];

    assert(http_build_request(req, sizeof(req), "/jsonrpc", "{}") == -1);
}

static void test_parse_status(void)
{
    static const char OK[] = "HTTP/1.0 200 OK\r\n\r\nbody";
    static const char NOTFOUND[] = "HTTP/1.1 404 Not Found\r\n\r\n";

    assert(http_parse_status(OK, strlen(OK)) == 200);
    assert(http_parse_status(NOTFOUND, strlen(NOTFOUND)) == 404);
    assert(http_parse_status("garbage", 7) == -1);
}

static void test_body_offset(void)
{
    static const char RESP[] = "HTTP/1.0 200 OK\r\nA: b\r\n\r\n{\"x\":1}";

    assert(http_body_offset(RESP, strlen(RESP)) == 25);
    assert(strcmp(RESP + 25, "{\"x\":1}") == 0);
    assert(http_body_offset("no separator", 12) == -1);
}

int main(void)
{
    test_build_request();
    test_build_truncation();
    test_parse_status();
    test_body_offset();
    printf("test_http_util: OK\n");
    return 0;
}
```

- [ ] **Step 2: Extend `tests/Makefile`** — add `test_http_util` to `TESTS`, append:

```make
test_http_util.exe: test_http_util.c ../src/http_util.c
	$(CC) $(CFLAGS) -o $@ $^
```

Run: expected FAIL.

- [ ] **Step 3: Create `include/http_util.h`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** HTTP/1.0 request building and response splitting helpers
*/

#ifndef HTTP_UTIL_H_
    #define HTTP_UTIL_H_

    #include <stddef.h>

int http_build_request(char *dst, size_t size, char const *path,
    char const *body);
int http_parse_status(char const *resp, size_t len);
long http_body_offset(char const *resp, size_t len);

#endif /* !HTTP_UTIL_H_ */
```

- [ ] **Step 4: Create `src/http_util.c`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** HTTP/1.0 request building and response splitting helpers
*/

#include <stdio.h>
#include <string.h>
#include "config.h"
#include "http_util.h"

int http_build_request(char *dst, size_t size, char const *path,
    char const *body)
{
    int written = snprintf(dst, size,
        "POST %s HTTP/1.0\r\n"
        "Host: %s\r\n"
        "Content-Type: application/json\r\n"
        "Content-Length: %u\r\n"
        "Connection: close\r\n\r\n%s",
        path, ODOO_HOST, (unsigned int)strlen(body), body);

    if (written < 0 || (size_t)written >= size)
        return -1;
    return written;
}

int http_parse_status(char const *resp, size_t len)
{
    int status = 0;

    if (len < 12 || strncmp(resp, "HTTP/1.", 7) != 0)
        return -1;
    for (size_t i = 9; i < 12; i++) {
        if (resp[i] < '0' || resp[i] > '9')
            return -1;
        status = status * 10 + (resp[i] - '0');
    }
    return status;
}

long http_body_offset(char const *resp, size_t len)
{
    for (size_t i = 0; i + 3 < len; i++) {
        if (memcmp(resp + i, "\r\n\r\n", 4) == 0)
            return (long)(i + 4);
    }
    return -1;
}
```

- [ ] **Step 5: Run** `make -C tests run` — Expected: all OK.
- [ ] **Step 6: Commit** — `feat: http/1.0 request building and response splitting`.

---

### Task 12: `refresh` — the refresh-decision policy

**Files:**
- Create: `include/refresh.h`, `src/refresh.c`, `tests/test_refresh.c`
- Modify: `tests/Makefile`

**Interfaces:**
- Consumes: `odoo.h`.
- Produces:
  - `struct snapshot { struct odoo_task_list list; unsigned int offline; };` — ALWAYS `memset` a snapshot to 0 before populating (memcmp compares padding too).
  - `struct refresh_times { uint32_t now_s; uint32_t last_refresh_s; unsigned int has_displayed; };`
  - `REFRESH_MIN_GAP_S` (180), `REFRESH_MAX_AGE_S` (86400)
  - `int refresh_needed(struct snapshot const *prev, struct snapshot const *cur, struct refresh_times const *t);`
  - Policy: first-ever display → 1; else `(changed || stale_24h) && spaced_3min`.

- [ ] **Step 1: Write the failing test `tests/test_refresh.c`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Unit tests for the panel refresh decision policy
*/

#include <assert.h>
#include <stdio.h>
#include <string.h>
#include "refresh.h"

static void test_first_display(void)
{
    struct snapshot a;
    struct snapshot b;
    struct refresh_times t = {30, 0, 0};

    memset(&a, 0, sizeof(a));
    memset(&b, 0, sizeof(b));
    assert(refresh_needed(&a, &b, &t) == 1);
}

static void test_unchanged_skips(void)
{
    struct snapshot a;
    struct refresh_times t = {10000, 5000, 1};

    memset(&a, 0, sizeof(a));
    assert(refresh_needed(&a, &a, &t) == 0);
}

static void test_change_triggers_when_spaced(void)
{
    struct snapshot a;
    struct snapshot b;
    struct refresh_times spaced = {1000, 500, 1};
    struct refresh_times close = {600, 500, 1};

    memset(&a, 0, sizeof(a));
    memset(&b, 0, sizeof(b));
    snprintf(b.list.tasks[0].name, sizeof(b.list.tasks[0].name), "X");
    b.list.count = 1;
    assert(refresh_needed(&a, &b, &spaced) == 1);
    assert(refresh_needed(&a, &b, &close) == 0);
}

static void test_daily_health_refresh(void)
{
    struct snapshot a;
    struct refresh_times t = {90000, 100, 1};

    memset(&a, 0, sizeof(a));
    assert(refresh_needed(&a, &a, &t) == 1);
}

static void test_offline_flip_triggers(void)
{
    struct snapshot a;
    struct snapshot b;
    struct refresh_times t = {1000, 500, 1};

    memset(&a, 0, sizeof(a));
    memset(&b, 0, sizeof(b));
    b.offline = 1;
    assert(refresh_needed(&a, &b, &t) == 1);
}

int main(void)
{
    test_first_display();
    test_unchanged_skips();
    test_change_triggers_when_spaced();
    test_daily_health_refresh();
    test_offline_flip_triggers();
    printf("test_refresh: OK\n");
    return 0;
}
```

- [ ] **Step 2: Extend `tests/Makefile`** — add `test_refresh` to `TESTS`, append:

```make
test_refresh.exe: test_refresh.c ../src/refresh.c
	$(CC) $(CFLAGS) -o $@ $^
```

Run: expected FAIL.

- [ ] **Step 3: Create `include/refresh.h`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Panel refresh decision policy: refresh only on change, daily, spaced
*/

#ifndef REFRESH_H_
    #define REFRESH_H_

    #include <stdint.h>
    #include "odoo.h"

#define REFRESH_MIN_GAP_S 180
#define REFRESH_MAX_AGE_S 86400

struct snapshot {
    struct odoo_task_list list;
    unsigned int offline;
};

struct refresh_times {
    uint32_t now_s;
    uint32_t last_refresh_s;
    unsigned int has_displayed;
};

int refresh_needed(struct snapshot const *prev, struct snapshot const *cur,
    struct refresh_times const *t);

#endif /* !REFRESH_H_ */
```

- [ ] **Step 4: Create `src/refresh.c`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Panel refresh decision policy: refresh only on change, daily, spaced
*/

#include <string.h>
#include "refresh.h"

int refresh_needed(struct snapshot const *prev, struct snapshot const *cur,
    struct refresh_times const *t)
{
    int changed = memcmp(prev, cur, sizeof(*prev)) != 0;
    int stale = (t->now_s - t->last_refresh_s) >= REFRESH_MAX_AGE_S;
    int spaced = (t->now_s - t->last_refresh_s) >= REFRESH_MIN_GAP_S;

    if (t->has_displayed == 0)
        return 1;
    return (changed || stale) && spaced;
}
```

- [ ] **Step 5: Run** `make -C tests run` — Expected: all OK.
- [ ] **Step 6: Commit** — `feat: refresh-on-change decision with daily and spacing rules`.

---

### Task 13: `dashboard` — layout and rendering

**Files:**
- Create: `include/dashboard.h`, `src/dashboard.c`, `tests/test_dashboard.c`
- Modify: `tests/Makefile`

**Interfaces:**
- Consumes: `gfx.h`, `time_fmt.h`, `refresh.h` (snapshot).
- Produces:
  - `struct dashboard_data { struct snapshot const *snap; struct tm today; char banner_date[16]; char updated_hhmm[8]; char offline_since[8]; };`
  - `void dashboard_render(uint8_t *fb, struct dashboard_data const *d);`
  - Layout: blue banner 0–59 (title scale 3, date scale 2, "N ouvertes" right-aligned); rows from y=70, 32 px each, up to 12 (red `*` if priority, name ≤24 chars scale 2 at x=32, project ≤9 chars at x=424, deadline DD/MM at x=576 colored by class, stage ≤8 chars at x=664, black separator line); footer y=458 (left slot: red `HORS LIGNE depuis HH:MM` if offline, else `+ d'autres taches` if overflow; right: `mise a jour HH:MM`); empty non-offline list → centered green `Aucune tache ouverte`.

- [ ] **Step 1: Write the failing test `tests/test_dashboard.c`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Unit tests for dashboard layout rendering
*/

#include <assert.h>
#include <stdio.h>
#include <string.h>
#include <time.h>
#include "dashboard.h"

static int px(uint8_t const *fb, int x, int y)
{
    size_t idx = ((size_t)y * GFX_WIDTH + (size_t)x) / 2;

    if ((x & 1) == 0)
        return fb[idx] >> 4;
    return fb[idx] & 0x0F;
}

static int region_has(uint8_t const *fb, struct gfx_rect const *r, int c)
{
    for (int y = r->y; y < r->y + r->h; y++) {
        for (int x = r->x; x < r->x + r->w; x++) {
            if (px(fb, x, y) == c)
                return 1;
        }
    }
    return 0;
}

static void fill_data(struct dashboard_data *d, struct snapshot *snap)
{
    memset(snap, 0, sizeof(*snap));
    memset(d, 0, sizeof(*d));
    d->snap = snap;
    d->today.tm_year = 126;
    d->today.tm_mon = 7;
    d->today.tm_mday = 26;
    snprintf(d->banner_date, sizeof(d->banner_date), "mer 26/08");
    snprintf(d->updated_hhmm, sizeof(d->updated_hhmm), "14:35");
}

static void test_banner_and_rows(void)
{
    static uint8_t fb[GFX_BUFFER_SIZE];
    static struct snapshot snap;
    struct dashboard_data d;
    struct gfx_rect star = {8, 70, 24, 32};
    struct gfx_rect date = {576, 70, 80, 32};

    fill_data(&d, &snap);
    snap.list.count = 1;
    snap.list.tasks[0].priority = 1;
    snprintf(snap.list.tasks[0].name, 64, "Tache urgente");
    snprintf(snap.list.tasks[0].deadline, 11, "2026-08-20");
    dashboard_render(fb, &d);
    assert(px(fb, 5, 5) == GFX_BLUE);
    assert(region_has(fb, &star, GFX_RED));
    assert(region_has(fb, &date, GFX_RED));
}

static void test_empty_state(void)
{
    static uint8_t fb[GFX_BUFFER_SIZE];
    static struct snapshot snap;
    struct dashboard_data d;
    struct gfx_rect middle = {0, 200, GFX_WIDTH, 120};

    fill_data(&d, &snap);
    dashboard_render(fb, &d);
    assert(region_has(fb, &middle, GFX_GREEN));
}

static void test_offline_footer(void)
{
    static uint8_t fb[GFX_BUFFER_SIZE];
    static struct snapshot snap;
    struct dashboard_data d;
    struct gfx_rect footer = {0, 450, GFX_WIDTH, 30};

    fill_data(&d, &snap);
    snap.offline = 1;
    snprintf(d.offline_since, sizeof(d.offline_since), "14:00");
    dashboard_render(fb, &d);
    assert(region_has(fb, &footer, GFX_RED));
}

static void test_deterministic(void)
{
    static uint8_t fb1[GFX_BUFFER_SIZE];
    static uint8_t fb2[GFX_BUFFER_SIZE];
    static struct snapshot snap;
    struct dashboard_data d;

    fill_data(&d, &snap);
    snap.list.count = 1;
    snprintf(snap.list.tasks[0].name, 64, "Stable");
    dashboard_render(fb1, &d);
    dashboard_render(fb2, &d);
    assert(memcmp(fb1, fb2, GFX_BUFFER_SIZE) == 0);
}

int main(void)
{
    test_banner_and_rows();
    test_empty_state();
    test_offline_footer();
    test_deterministic();
    printf("test_dashboard: OK\n");
    return 0;
}
```

- [ ] **Step 2: Extend `tests/Makefile`** — add `test_dashboard` to `TESTS`, append:

```make
test_dashboard.exe: test_dashboard.c ../src/dashboard.c ../src/gfx.c \
	../src/gfx_text.c ../src/time_fmt.c
	$(CC) $(CFLAGS) -o $@ $^
```

Run: expected FAIL.

- [ ] **Step 3: Create `include/dashboard.h`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Dashboard layout rendering of the task snapshot into the framebuffer
*/

#ifndef DASHBOARD_H_
    #define DASHBOARD_H_

    #include <time.h>
    #include "gfx.h"
    #include "refresh.h"

#define DASH_BANNER_H 60
#define DASH_ROWS_TOP 70
#define DASH_ROW_H 32
#define DASH_FOOTER_Y 458

struct dashboard_data {
    struct snapshot const *snap;
    struct tm today;
    char banner_date[16];
    char updated_hhmm[8];
    char offline_since[8];
};

void dashboard_render(uint8_t *fb, struct dashboard_data const *d);

#endif /* !DASHBOARD_H_ */
```

- [ ] **Step 4: Create `src/dashboard.c`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Dashboard layout rendering of the task snapshot into the framebuffer
*/

#include <stdio.h>
#include "dashboard.h"
#include "time_fmt.h"

static void draw_banner(uint8_t *fb, struct dashboard_data const *d)
{
    struct gfx_rect bar = {0, 0, GFX_WIDTH, DASH_BANNER_H};
    struct gfx_style title = {16, 18, GFX_WHITE, 3};
    struct gfx_style date = {360, 22, GFX_WHITE, 2};
    struct gfx_style count = {0, 22, GFX_WHITE, 2};
    char text[24];

    gfx_fill_rect(fb, &bar, GFX_BLUE);
    gfx_text(fb, &title, "MES TACHES");
    gfx_text(fb, &date, d->banner_date);
    snprintf(text, sizeof(text), "%u ouvertes", d->snap->list.count);
    count.x = GFX_WIDTH - 16 - gfx_text_width(text, 2);
    gfx_text(fb, &count, text);
}

static int deadline_color(struct odoo_task const *task,
    struct tm const *today)
{
    int cls = time_fmt_deadline_class(task->deadline, today);

    if (cls == DL_OVERDUE)
        return GFX_RED;
    if (cls == DL_SOON)
        return GFX_BLUE;
    return GFX_BLACK;
}

static void draw_row(uint8_t *fb, struct odoo_task const *task, int y,
    struct tm const *today)
{
    struct gfx_style star = {8, y + 8, GFX_RED, 2};
    struct gfx_style name = {32, y + 8, GFX_BLACK, 2};
    struct gfx_style proj = {424, y + 8, GFX_BLACK, 2};
    struct gfx_style date = {576, y + 8, deadline_color(task, today), 2};
    struct gfx_style stage = {664, y + 8, GFX_BLACK, 2};
    char field[32];

    if (task->priority > 0)
        gfx_text(fb, &star, "*");
    snprintf(field, sizeof(field), "%.24s", task->name);
    gfx_text(fb, &name, field);
    snprintf(field, sizeof(field), "%.9s", task->project);
    gfx_text(fb, &proj, field);
    time_fmt_ddmm(field, sizeof(field), task->deadline);
    gfx_text(fb, &date, field);
    snprintf(field, sizeof(field), "%.8s", task->stage);
    gfx_text(fb, &stage, field);
}

static void draw_rows(uint8_t *fb, struct dashboard_data const *d)
{
    struct gfx_rect sep = {8, 0, GFX_WIDTH - 16, 1};
    int y = DASH_ROWS_TOP;

    for (unsigned int i = 0; i < d->snap->list.count; i++) {
        draw_row(fb, &d->snap->list.tasks[i], y, &d->today);
        sep.y = y + DASH_ROW_H - 1;
        gfx_fill_rect(fb, &sep, GFX_BLACK);
        y += DASH_ROW_H;
    }
}

static void draw_footer(uint8_t *fb, struct dashboard_data const *d)
{
    struct gfx_style left = {8, DASH_FOOTER_Y, GFX_BLACK, 2};
    struct gfx_style right = {0, DASH_FOOTER_Y, GFX_BLACK, 2};
    char text[48];

    snprintf(text, sizeof(text), "mise a jour %s", d->updated_hhmm);
    right.x = GFX_WIDTH - 8 - gfx_text_width(text, 2);
    gfx_text(fb, &right, text);
    if (d->snap->offline != 0) {
        left.color = GFX_RED;
        snprintf(text, sizeof(text), "HORS LIGNE depuis %s",
            d->offline_since);
        gfx_text(fb, &left, text);
        return;
    }
    if (d->snap->list.overflow != 0)
        gfx_text(fb, &left, "+ d'autres taches");
}

static void draw_empty(uint8_t *fb)
{
    struct gfx_style style = {0, 220, GFX_GREEN, 3};

    gfx_text_centered(fb, &style, "Aucune tache ouverte");
}

void dashboard_render(uint8_t *fb, struct dashboard_data const *d)
{
    gfx_fill(fb, GFX_WHITE);
    draw_banner(fb, d);
    if (d->snap->list.count == 0)
        draw_empty(fb);
    else
        draw_rows(fb, d);
    draw_footer(fb, d);
}
```

- [ ] **Step 5: Run** `make -C tests run` — Expected: all OK.
- [ ] **Step 6: Commit** — `feat: render task dashboard layout with status footer`.

---

### Task 14: `epd_io` + `epd_driver` — panel driver port

**Files:**
- Create: `include/epd.h`, `src/epd_io.c`, `src/epd_driver.c`
- Modify: `CMakeLists.txt`

**Interfaces:**
- Consumes: `config.h` (`EPD_CLK_HALF_PERIOD_US`), `gfx.h` (`GFX_BUFFER_SIZE`), Pico SDK gpio/time/watchdog. Port of validated `micropython/main.py` — same pins, same init bytes, same BUSY semantics (BUSY low = busy).
- Produces: `void epd_io_init(void);` `void epd_io_reset(void);` `void epd_io_command(uint8_t cmd);` `void epd_io_data(uint8_t const *data, size_t len);` `int epd_io_wait_idle(uint32_t timeout_ms);` `int epd_init(void);` `int epd_display(uint8_t const *fb);` `void epd_sleep(void);` — per-refresh flow used by main: `epd_init()` → `epd_display(fb)` → `epd_sleep()` (always, even on failure).

- [ ] **Step 1: Create `include/epd.h`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Waveshare 7.3 inch e-Paper HAT (E) Spectra 6 driver over paced bit-bang
*/

#ifndef EPD_H_
    #define EPD_H_

    #include <stddef.h>
    #include <stdint.h>

    #ifndef EPD_CLK_HALF_PERIOD_US
        #define EPD_CLK_HALF_PERIOD_US 1
    #endif

#define EPD_RST_PIN 12
#define EPD_DC_PIN 8
#define EPD_CS_PIN 9
#define EPD_BUSY_PIN 13
#define EPD_CLK_PIN 10
#define EPD_DIN_PIN 11

struct epd_cmd {
    uint8_t cmd;
    uint8_t len;
    uint8_t data[6];
};

void epd_io_init(void);
void epd_io_reset(void);
void epd_io_command(uint8_t cmd);
void epd_io_data(uint8_t const *data, size_t len);
int epd_io_wait_idle(uint32_t timeout_ms);
int epd_init(void);
int epd_display(uint8_t const *fb);
void epd_sleep(void);

#endif /* !EPD_H_ */
```

- [ ] **Step 2: Create `src/epd_io.c`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Paced bit-bang SPI transport for the e-paper panel (mode 0, MSB first)
*/

#include "pico/stdlib.h"
#include "hardware/gpio.h"
#include "hardware/watchdog.h"
#include "config.h"
#include "epd.h"

void epd_io_init(void)
{
    gpio_init(EPD_RST_PIN);
    gpio_init(EPD_DC_PIN);
    gpio_init(EPD_CS_PIN);
    gpio_init(EPD_CLK_PIN);
    gpio_init(EPD_DIN_PIN);
    gpio_init(EPD_BUSY_PIN);
    gpio_set_dir(EPD_RST_PIN, GPIO_OUT);
    gpio_set_dir(EPD_DC_PIN, GPIO_OUT);
    gpio_set_dir(EPD_CS_PIN, GPIO_OUT);
    gpio_set_dir(EPD_CLK_PIN, GPIO_OUT);
    gpio_set_dir(EPD_DIN_PIN, GPIO_OUT);
    gpio_set_dir(EPD_BUSY_PIN, GPIO_IN);
    gpio_pull_up(EPD_BUSY_PIN);
    gpio_put(EPD_CS_PIN, 1);
    gpio_put(EPD_CLK_PIN, 0);
}

void epd_io_reset(void)
{
    gpio_put(EPD_RST_PIN, 1);
    sleep_ms(20);
    gpio_put(EPD_RST_PIN, 0);
    sleep_ms(2);
    gpio_put(EPD_RST_PIN, 1);
    sleep_ms(20);
}

static void tx_byte(uint8_t byte)
{
    for (int bit = 7; bit >= 0; bit--) {
        gpio_put(EPD_DIN_PIN, (byte >> bit) & 1);
        busy_wait_us(EPD_CLK_HALF_PERIOD_US);
        gpio_put(EPD_CLK_PIN, 1);
        busy_wait_us(EPD_CLK_HALF_PERIOD_US);
        gpio_put(EPD_CLK_PIN, 0);
    }
}

void epd_io_command(uint8_t cmd)
{
    gpio_put(EPD_DC_PIN, 0);
    gpio_put(EPD_CS_PIN, 0);
    tx_byte(cmd);
    gpio_put(EPD_CS_PIN, 1);
}

void epd_io_data(uint8_t const *data, size_t len)
{
    gpio_put(EPD_DC_PIN, 1);
    gpio_put(EPD_CS_PIN, 0);
    for (size_t i = 0; i < len; i++) {
        tx_byte(data[i]);
        if ((i & 0xFF) == 0)
            watchdog_update();
    }
    gpio_put(EPD_CS_PIN, 1);
}

int epd_io_wait_idle(uint32_t timeout_ms)
{
    absolute_time_t deadline = make_timeout_time_ms(timeout_ms);

    sleep_ms(30);
    while (gpio_get(EPD_BUSY_PIN) == 0) {
        if (absolute_time_diff_us(get_absolute_time(), deadline) < 0)
            return -1;
        watchdog_update();
        sleep_ms(10);
    }
    return 0;
}
```

- [ ] **Step 3: Create `src/epd_driver.c`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Panel init sequence, refresh cycle and deep sleep (port of main.py)
*/

#include "pico/stdlib.h"
#include "hardware/gpio.h"
#include "hardware/watchdog.h"
#include "epd.h"
#include "gfx.h"

static const struct epd_cmd INIT_SEQUENCE[] = {
    {0xAA, 6, {0x49, 0x55, 0x20, 0x08, 0x09, 0x18}},
    {0x01, 1, {0x3F, 0, 0, 0, 0, 0}},
    {0x00, 2, {0x5F, 0x69, 0, 0, 0, 0}},
    {0x03, 4, {0x00, 0x54, 0x00, 0x44, 0, 0}},
    {0x05, 4, {0x40, 0x1F, 0x1F, 0x2C, 0, 0}},
    {0x06, 4, {0x6F, 0x1F, 0x17, 0x49, 0, 0}},
    {0x08, 4, {0x6F, 0x1F, 0x1F, 0x22, 0, 0}},
    {0x30, 1, {0x03, 0, 0, 0, 0, 0}},
    {0x50, 1, {0x3F, 0, 0, 0, 0, 0}},
    {0x60, 2, {0x02, 0x00, 0, 0, 0, 0}},
    {0x61, 4, {0x03, 0x20, 0x01, 0xE0, 0, 0}},
    {0x84, 1, {0x01, 0, 0, 0, 0, 0}},
    {0xE3, 1, {0x2F, 0, 0, 0, 0, 0}},
};

static void send_init_sequence(void)
{
    size_t total = sizeof(INIT_SEQUENCE) / sizeof(INIT_SEQUENCE[0]);

    for (size_t i = 0; i < total; i++) {
        epd_io_command(INIT_SEQUENCE[i].cmd);
        epd_io_data(INIT_SEQUENCE[i].data, INIT_SEQUENCE[i].len);
    }
}

static int power_on_reacts(void)
{
    absolute_time_t deadline = make_timeout_time_ms(3000);

    epd_io_command(0x04);
    while (absolute_time_diff_us(get_absolute_time(), deadline) > 0) {
        if (gpio_get(EPD_BUSY_PIN) == 0)
            return epd_io_wait_idle(60000);
        watchdog_update();
        sleep_ms(2);
    }
    return -1;
}

int epd_init(void)
{
    for (int attempt = 0; attempt < 3; attempt++) {
        epd_io_reset();
        epd_io_wait_idle(5000);
        send_init_sequence();
        if (power_on_reacts() == 0)
            return 0;
        sleep_ms(200);
    }
    return -1;
}

static int turn_on_display(void)
{
    uint8_t zero = 0x00;

    epd_io_command(0x04);
    if (epd_io_wait_idle(60000) != 0)
        return -1;
    epd_io_command(0x12);
    epd_io_data(&zero, 1);
    if (epd_io_wait_idle(60000) != 0)
        return -1;
    epd_io_command(0x02);
    epd_io_data(&zero, 1);
    return epd_io_wait_idle(60000);
}

int epd_display(uint8_t const *fb)
{
    epd_io_command(0x10);
    epd_io_data(fb, GFX_BUFFER_SIZE);
    return turn_on_display();
}

void epd_sleep(void)
{
    uint8_t code = 0xA5;

    epd_io_command(0x07);
    epd_io_data(&code, 1);
    sleep_ms(20);
}
```

- [ ] **Step 4: Extend `CMakeLists.txt`** — in `add_executable`, the source list becomes:

```cmake
add_executable(epaper_dashboard
    src/main.c
    src/epd_io.c
    src/epd_driver.c
    src/gfx.c
    src/gfx_text.c
    src/json_str.c
    src/jsmn_util.c
    src/odoo_parse.c
    src/odoo_request.c
    src/time_fmt.c
    src/http_util.c
    src/refresh.c
    src/dashboard.c
)
```

and `target_link_libraries` gains `hardware_watchdog`. These files reference `config.h` — add the config gate right after `project(...)`:

```cmake
if(NOT EXISTS ${CMAKE_CURRENT_LIST_DIR}/include/config.h)
    message(FATAL_ERROR
        "include/config.h missing: copy config.h.example there first")
endif()
```

Then create a local build config: `cp config.h.example include/config.h` (dummy values are fine for compiling; verify `git status` shows it untracked).

- [ ] **Step 5: Build** — `cmake -S . -B build -G Ninja && cmake --build build`. Expected: `.uf2` produced, no warnings in our files.

- [ ] **Step 6: Commit** — `feat: port validated e-paper driver to paced bit-bang C` (config.h must NOT be in the commit).

---

### Task 15: WiFi bring-up

**Files:**
- Create: `lwipopts.h`, `include/net_wifi.h`, `src/net_wifi.c`, `include/sys_idle.h`, `src/sys_idle.c`
- Modify: `CMakeLists.txt`, `src/main.c` (temporary wifi-demo main, replaced in Task 19)

**Interfaces:**
- Consumes: `config.h` (`WIFI_SSID`, `WIFI_PASSWORD`, `WIFI_COUNTRY`), Task 1 SDK.
- Produces: `int net_wifi_init(void);` `int net_wifi_connect(void);` (blocking ≤30 s, watchdog-fed, -1 on fail) `int net_wifi_up(void);` `void net_wifi_led(int on);` `void sys_idle_ms(uint32_t ms);` (sleep slices + `cyw43_arch_poll` + `watchdog_update`); `lwipopts.h` used by all later network tasks.

- [ ] **Step 1: Create `lwipopts.h`** — clone examples and merge (Bash):

```bash
git clone --depth 1 https://github.com/raspberrypi/pico-examples.git \
  "$TMPDIR_SCRATCH/pico-examples" 2>/dev/null || true
cp "$TMPDIR_SCRATCH/pico-examples/pico_w/wifi/lwipopts_examples_common.h" lwipopts.h
```

(Use the session scratchpad directory for the clone.) Then edit `lwipopts.h`: change its include guard to `LWIPOPTS_H_` if needed, and append **before** the final `#endif`:

```c
/* project additions: sntp (Task 16) and altcp tls (Task 17) land here */
```

Keep the file otherwise verbatim (it is derived config, style-exempt like vendored code, but keep LF endings).

- [ ] **Step 2: Create `include/sys_idle.h`** and **`src/sys_idle.c`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Watchdog-fed idle wait that keeps the wifi driver polled
*/

#ifndef SYS_IDLE_H_
    #define SYS_IDLE_H_

    #include <stdint.h>

void sys_idle_ms(uint32_t ms);

#endif /* !SYS_IDLE_H_ */
```

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Watchdog-fed idle wait that keeps the wifi driver polled
*/

#include "pico/stdlib.h"
#include "pico/cyw43_arch.h"
#include "hardware/watchdog.h"
#include "sys_idle.h"

void sys_idle_ms(uint32_t ms)
{
    absolute_time_t deadline = make_timeout_time_ms(ms);

    while (absolute_time_diff_us(get_absolute_time(), deadline) > 0) {
        cyw43_arch_poll();
        watchdog_update();
        sleep_ms(10);
    }
}
```

- [ ] **Step 3: Create `include/net_wifi.h`** and **`src/net_wifi.c`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** WiFi station bring-up, reconnection and status
*/

#ifndef NET_WIFI_H_
    #define NET_WIFI_H_

int net_wifi_init(void);
int net_wifi_connect(void);
int net_wifi_up(void);
void net_wifi_led(int on);

#endif /* !NET_WIFI_H_ */
```

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** WiFi station bring-up, reconnection and status
*/

#include "pico/stdlib.h"
#include "pico/cyw43_arch.h"
#include "hardware/watchdog.h"
#include "config.h"
#include "net_wifi.h"

int net_wifi_init(void)
{
    if (cyw43_arch_init_with_country(WIFI_COUNTRY) != 0)
        return -1;
    cyw43_arch_enable_sta_mode();
    return 0;
}

int net_wifi_connect(void)
{
    absolute_time_t deadline = make_timeout_time_ms(30000);
    int status = 0;

    if (cyw43_arch_wifi_connect_async(WIFI_SSID, WIFI_PASSWORD,
        CYW43_AUTH_WPA2_AES_PSK) != 0)
        return -1;
    while (absolute_time_diff_us(get_absolute_time(), deadline) > 0) {
        cyw43_arch_poll();
        watchdog_update();
        status = cyw43_tcpip_link_status(&cyw43_state, CYW43_ITF_STA);
        if (status == CYW43_LINK_UP)
            return 0;
        if (status < 0)
            return -1;
        sleep_ms(50);
    }
    return -1;
}

int net_wifi_up(void)
{
    return cyw43_tcpip_link_status(&cyw43_state, CYW43_ITF_STA)
        == CYW43_LINK_UP;
}

void net_wifi_led(int on)
{
    cyw43_arch_gpio_put(CYW43_WL_GPIO_LED_PIN, on != 0);
}
```

- [ ] **Step 4: Replace `src/main.c` with the wifi demo** (temporary — Task 19 replaces it):

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Entry point (wifi bring-up demo, replaced by the real superloop later)
*/

#include <stdio.h>
#include "pico/stdlib.h"
#include "pico/cyw43_arch.h"
#include "hardware/watchdog.h"
#include "net_wifi.h"
#include "sys_idle.h"

int main(void)
{
    stdio_init_all();
    watchdog_enable(8000, 1);
    if (net_wifi_init() != 0) {
        printf("wifi: init failed\n");
        return 1;
    }
    while (net_wifi_connect() != 0)
        printf("wifi: connect failed, retrying\n");
    printf("wifi: connected\n");
    for (;;) {
        net_wifi_led(1);
        sys_idle_ms(500);
        net_wifi_led(0);
        sys_idle_ms(500);
    }
    return 0;
}
```

- [ ] **Step 5: Extend `CMakeLists.txt`** — add `src/net_wifi.c` and `src/sys_idle.c` to the source list; change `target_link_libraries` to:

```cmake
target_link_libraries(epaper_dashboard
    pico_stdlib
    pico_cyw43_arch_lwip_poll
    hardware_watchdog
)
```

- [ ] **Step 6: Build** — expected: `.uf2` builds.
- [ ] **Step 7 [HW]:** ask user to flash; serial should show `wifi: connected` and the onboard LED blinks. Continue regardless.
- [ ] **Step 8: Commit** — `feat: wifi station bring-up with watchdog-fed polling`.

---

### Task 16: SNTP time sync

**Files:**
- Create: `include/net_time.h`, `src/net_time.c`
- Modify: `lwipopts.h`, `CMakeLists.txt`, `src/main.c` (add time log line to demo main)

**Interfaces:**
- Consumes: lwIP SNTP app, `pico_aon_timer`, `config.h` (`NTP_SERVER`, `TZ_OFFSET_MIN`).
- Produces: `void net_time_init(void);` `int net_time_synced(void);` `void net_time_local(struct tm *out);` `void net_time_sntp_set(unsigned int sec);` (called by lwIP via macro — do not call manually).

- [ ] **Step 1: Append the SNTP block to `lwipopts.h`** (before final `#endif`):

```c
#define SNTP_SERVER_DNS 1
#define SNTP_STARTUP_DELAY 0
extern void net_time_sntp_set(unsigned int sec);
#define SNTP_SET_SYSTEM_TIME(sec) net_time_sntp_set(sec)
```

- [ ] **Step 2: Create `include/net_time.h`** and **`src/net_time.c`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** SNTP time synchronisation backed by the always-on timer
*/

#ifndef NET_TIME_H_
    #define NET_TIME_H_

    #include <time.h>

void net_time_init(void);
int net_time_synced(void);
void net_time_local(struct tm *out);
void net_time_sntp_set(unsigned int sec);

#endif /* !NET_TIME_H_ */
```

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** SNTP time synchronisation backed by the always-on timer
*/

#include "pico/aon_timer.h"
#include "lwip/apps/sntp.h"
#include "config.h"
#include "net_time.h"

void net_time_init(void)
{
    sntp_setoperatingmode(SNTP_OPMODE_POLL);
    sntp_setservername(0, NTP_SERVER);
    sntp_init();
}

void net_time_sntp_set(unsigned int sec)
{
    struct timespec ts = {(time_t)sec, 0};

    if (aon_timer_is_running())
        aon_timer_set_time(&ts);
    else
        aon_timer_start(&ts);
}

int net_time_synced(void)
{
    return aon_timer_is_running() ? 1 : 0;
}

void net_time_local(struct tm *out)
{
    struct timespec ts = {0, 0};
    time_t local = 0;

    aon_timer_get_time(&ts);
    local = ts.tv_sec + TZ_OFFSET_MIN * 60;
    gmtime_r(&local, out);
}
```

- [ ] **Step 3: Wire into the demo main** — after `wifi: connected`, add:

```c
    net_time_init();
    while (net_time_synced() == 0)
        sys_idle_ms(500);
    printf("time: synced\n");
```

(with `#include "net_time.h"` on top; keep `main` ≤ 20 lines by removing the LED blink loop body to a single `sys_idle_ms(1000);` if needed).

- [ ] **Step 4: Extend `CMakeLists.txt`** — add `src/net_time.c` to sources; add `pico_lwip_sntp` and `pico_aon_timer` to `target_link_libraries`.
- [ ] **Step 5: Build** — expected `.uf2` builds. If `pico_aon_timer` is unknown in the chosen SDK tag, use `hardware_rtc`-equivalent `aon_timer` naming from that SDK's docs (`ls $PICO_SDK_PATH/src/common | grep -i aon`) and adapt.
- [ ] **Step 6 [HW]:** user flash → serial shows `time: synced`. Continue regardless.
- [ ] **Step 7: Commit** — `feat: sntp time sync stored in the always-on timer`.

---

### Task 17: TLS HTTP client

**Files:**
- Create: `mbedtls_config.h`, `include/http_client.h`, `src/http_conn.c`, `src/http_client.c`
- Modify: `lwipopts.h`, `CMakeLists.txt`

**Interfaces:**
- Consumes: `http_util` (Task 11), lwIP altcp_tls + mbedTLS, `config.h` (`ODOO_HOST`, `ODOO_PORT`, optional `ODOO_CA_CERT`).
- Produces:
  - `struct http_ctx { struct altcp_pcb *pcb; char const *request; size_t request_len; char *resp; size_t resp_cap; size_t resp_len; ip_addr_t addr; int phase; };` (phase: 0 dns-wait, 1 resolved, 2 sent, 3 done, <0 error)
  - `struct http_response { int status; char const *body; size_t body_len; };`
  - `int http_conn_resolve(struct http_ctx *ctx);` `int http_conn_open(struct http_ctx *ctx, struct altcp_tls_config *tls);` `int http_conn_wait(struct http_ctx *ctx, int target, uint32_t timeout_ms);` `void http_conn_close(struct http_ctx *ctx);`
  - `int http_post_json(char const *path, char const *body, struct http_response *out);` — 0 ok / -1 network / -2 bad status or malformed. Body points into a static buffer (valid until next call) and is NUL-terminated.
  - `HTTP_RESP_CAP` 16384, `HTTP_REQ_CAP` 4096.

- [ ] **Step 1: Copy `mbedtls_config.h`** from the examples clone (Bash):

```bash
cp "$TMPDIR_SCRATCH/pico-examples/pico_w/wifi/tls_client/mbedtls_config.h" mbedtls_config.h
grep -c MBEDTLS_SSL_CLI_C mbedtls_config.h
```

Expected: `1`. Verify `MBEDTLS_HAVE_TIME` is NOT defined in it (if it is, remove it — cert time validity is deliberately unchecked, see Global Constraints deviation 1). Keep the file otherwise verbatim.

- [ ] **Step 2: Append the ALTCP/TLS block to `lwipopts.h`** (before final `#endif`):

```c
#define LWIP_ALTCP 1
#define LWIP_ALTCP_TLS 1
#define LWIP_ALTCP_TLS_MBEDTLS 1
#define MEM_SIZE 8000
```

(If the copied file already defines `MEM_SIZE`, replace its value with 8000 instead of redefining.)

- [ ] **Step 3: Create `include/http_client.h`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** One-shot HTTPS POST client over lwIP altcp_tls
*/

#ifndef HTTP_CLIENT_H_
    #define HTTP_CLIENT_H_

    #include <stddef.h>
    #include "lwip/altcp.h"
    #include "lwip/altcp_tls.h"
    #include "lwip/ip_addr.h"

#define HTTP_RESP_CAP 16384
#define HTTP_REQ_CAP 4096

struct http_ctx {
    struct altcp_pcb *pcb;
    char const *request;
    size_t request_len;
    char *resp;
    size_t resp_cap;
    size_t resp_len;
    ip_addr_t addr;
    int phase;
};

struct http_response {
    int status;
    char const *body;
    size_t body_len;
};

int http_conn_resolve(struct http_ctx *ctx);
int http_conn_open(struct http_ctx *ctx, struct altcp_tls_config *tls);
int http_conn_wait(struct http_ctx *ctx, int target, uint32_t timeout_ms);
void http_conn_close(struct http_ctx *ctx);
int http_post_json(char const *path, char const *body,
    struct http_response *out);

#endif /* !HTTP_CLIENT_H_ */
```

- [ ] **Step 4: Create `src/http_conn.c`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Connection lifecycle and lwIP callbacks for the HTTPS client
*/

#include "pico/stdlib.h"
#include "pico/cyw43_arch.h"
#include "hardware/watchdog.h"
#include "lwip/dns.h"
#include "lwip/pbuf.h"
#include "mbedtls/ssl.h"
#include "config.h"
#include "http_client.h"

static void cb_dns(char const *name, ip_addr_t const *addr, void *arg)
{
    struct http_ctx *ctx = arg;

    (void)name;
    if (addr == 0) {
        ctx->phase = -1;
        return;
    }
    ctx->addr = *addr;
    ctx->phase = 1;
}

static err_t cb_connected(void *arg, struct altcp_pcb *pcb, err_t err)
{
    struct http_ctx *ctx = arg;

    if (err != ERR_OK) {
        ctx->phase = -2;
        return ERR_OK;
    }
    altcp_write(pcb, ctx->request, (u16_t)ctx->request_len, 0);
    altcp_output(pcb);
    ctx->phase = 2;
    return ERR_OK;
}

static err_t cb_recv(void *arg, struct altcp_pcb *pcb, struct pbuf *p,
    err_t err)
{
    struct http_ctx *ctx = arg;
    u16_t room = (u16_t)(ctx->resp_cap - ctx->resp_len - 1);
    u16_t copied = 0;

    (void)err;
    if (p == 0) {
        ctx->phase = 3;
        return ERR_OK;
    }
    copied = pbuf_copy_partial(p, ctx->resp + ctx->resp_len, room, 0);
    ctx->resp_len += copied;
    if (copied < p->tot_len)
        ctx->phase = -3;
    altcp_recved(pcb, p->tot_len);
    pbuf_free(p);
    return ERR_OK;
}

static void cb_err(void *arg, err_t err)
{
    struct http_ctx *ctx = arg;

    (void)err;
    ctx->pcb = 0;
    if (ctx->phase != 3)
        ctx->phase = -4;
}

int http_conn_resolve(struct http_ctx *ctx)
{
    err_t err = dns_gethostbyname(ODOO_HOST, &ctx->addr, cb_dns, ctx);

    if (err == ERR_OK) {
        ctx->phase = 1;
        return 0;
    }
    if (err != ERR_INPROGRESS)
        return -1;
    return http_conn_wait(ctx, 1, 10000);
}

int http_conn_open(struct http_ctx *ctx, struct altcp_tls_config *tls)
{
    ctx->pcb = altcp_tls_new(tls, IPADDR_TYPE_V4);
    if (ctx->pcb == 0)
        return -1;
    mbedtls_ssl_set_hostname(altcp_tls_context(ctx->pcb), ODOO_HOST);
    altcp_arg(ctx->pcb, ctx);
    altcp_err(ctx->pcb, cb_err);
    altcp_recv(ctx->pcb, cb_recv);
    if (altcp_connect(ctx->pcb, &ctx->addr, ODOO_PORT, cb_connected)
        != ERR_OK)
        return -1;
    return 0;
}

int http_conn_wait(struct http_ctx *ctx, int target, uint32_t timeout_ms)
{
    absolute_time_t deadline = make_timeout_time_ms(timeout_ms);

    while (ctx->phase >= 0 && ctx->phase < target) {
        if (absolute_time_diff_us(get_absolute_time(), deadline) < 0)
            return -1;
        cyw43_arch_poll();
        watchdog_update();
        sleep_ms(5);
    }
    return ctx->phase >= target ? 0 : -1;
}

void http_conn_close(struct http_ctx *ctx)
{
    if (ctx->pcb == 0)
        return;
    altcp_arg(ctx->pcb, 0);
    if (altcp_close(ctx->pcb) != ERR_OK)
        altcp_abort(ctx->pcb);
    ctx->pcb = 0;
}
```

- [ ] **Step 5: Create `src/http_client.c`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** One-shot HTTPS POST client over lwIP altcp_tls
*/

#include <string.h>
#include "config.h"
#include "http_client.h"
#include "http_util.h"

#ifdef ODOO_CA_CERT
    #define ODOO_CA_CERT_PTR ((u8_t const *)ODOO_CA_CERT)
    #define ODOO_CA_CERT_LEN (sizeof(ODOO_CA_CERT))
#else
    #define ODOO_CA_CERT_PTR 0
    #define ODOO_CA_CERT_LEN 0
#endif

static struct altcp_tls_config *make_tls_config(void)
{
    return altcp_tls_create_config_client(ODOO_CA_CERT_PTR,
        ODOO_CA_CERT_LEN);
}

static int parse_response(struct http_ctx *ctx, struct http_response *out)
{
    long body_off = 0;

    ctx->resp[ctx->resp_len] = '\0';
    body_off = http_body_offset(ctx->resp, ctx->resp_len);
    out->status = http_parse_status(ctx->resp, ctx->resp_len);
    if (out->status != 200 || body_off < 0)
        return -2;
    out->body = ctx->resp + body_off;
    out->body_len = ctx->resp_len - (size_t)body_off;
    return 0;
}

static int finish(struct http_ctx *ctx, struct altcp_tls_config *tls,
    int code)
{
    http_conn_close(ctx);
    if (tls != 0)
        altcp_tls_free_config(tls);
    return code;
}

int http_post_json(char const *path, char const *body,
    struct http_response *out)
{
    static char resp[HTTP_RESP_CAP];
    static char req[HTTP_REQ_CAP];
    struct http_ctx ctx = {0, req, 0, resp, HTTP_RESP_CAP, 0, {0}, 0};
    struct altcp_tls_config *tls = make_tls_config();
    int built = http_build_request(req, sizeof(req), path, body);

    if (tls == 0 || built < 0)
        return finish(&ctx, tls, -1);
    ctx.request_len = (size_t)built;
    if (http_conn_resolve(&ctx) != 0 || http_conn_open(&ctx, tls) != 0)
        return finish(&ctx, tls, -1);
    if (http_conn_wait(&ctx, 3, 30000) != 0)
        return finish(&ctx, tls, -1);
    return finish(&ctx, tls, parse_response(&ctx, out));
}
```

- [ ] **Step 6: Extend `CMakeLists.txt`** — add `src/http_conn.c` and `src/http_client.c` to sources; `target_link_libraries` gains `pico_lwip_mbedtls` and `pico_mbedtls`.
- [ ] **Step 7: Build** — expected `.uf2` builds. Known friction points: `altcp_tls_context()` needs `lwip/altcp_tls.h` (already included); if `mbedtls_ssl_set_hostname` is undeclared, include `mbedtls/ssl.h` (http_conn.c does). Fix include/lib issues here — do not change the module interfaces.
- [ ] **Step 8: Commit** — `feat: https post client over altcp_tls with optional ca pinning`.

---

### Task 18: `odoo_client` — auth + fetch orchestration

**Files:**
- Create: `include/odoo_client.h`, `src/odoo_client.c`
- Modify: `CMakeLists.txt`

**Interfaces:**
- Consumes: `odoo_request`, `odoo_parse`, `http_client`.
- Produces: `int odoo_client_sync(int *uid, struct odoo_task_list *list);` — authenticates when `*uid <= 0`, fetches tasks, re-authenticates+retries once when Odoo returns an error object (-2). Returns 0 ok / -1 failed.

- [ ] **Step 1: Create `include/odoo_client.h`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Odoo sync orchestration: authenticate, fetch tasks, re-auth on error
*/

#ifndef ODOO_CLIENT_H_
    #define ODOO_CLIENT_H_

    #include "odoo.h"

int odoo_client_sync(int *uid, struct odoo_task_list *list);

#endif /* !ODOO_CLIENT_H_ */
```

- [ ] **Step 2: Create `src/odoo_client.c`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Odoo sync orchestration: authenticate, fetch tasks, re-auth on error
*/

#include <stdio.h>
#include "http_client.h"
#include "odoo_client.h"
#include "odoo_parse.h"
#include "odoo_request.h"

static int do_auth(int *uid)
{
    static char body[ODOO_REQ_CAP];
    struct http_response resp = {0, 0, 0};

    if (odoo_build_auth(body, sizeof(body)) < 0)
        return -1;
    if (http_post_json("/jsonrpc", body, &resp) != 0)
        return -1;
    if (odoo_parse_auth(resp.body, resp.body_len, uid) != 0)
        return -1;
    printf("odoo: authenticated uid=%d\n", *uid);
    return 0;
}

static int do_fetch(int uid, struct odoo_task_list *list)
{
    static char body[ODOO_REQ_CAP];
    struct http_response resp = {0, 0, 0};

    if (odoo_build_tasks(body, sizeof(body), uid) < 0)
        return -1;
    if (http_post_json("/jsonrpc", body, &resp) != 0)
        return -1;
    return odoo_parse_tasks(resp.body, resp.body_len, list);
}

int odoo_client_sync(int *uid, struct odoo_task_list *list)
{
    int ret = 0;

    if (*uid <= 0 && do_auth(uid) != 0)
        return -1;
    ret = do_fetch(*uid, list);
    if (ret == -2 && do_auth(uid) == 0)
        ret = do_fetch(*uid, list);
    return ret == 0 ? 0 : -1;
}
```

- [ ] **Step 3: Extend `CMakeLists.txt`** — add `src/odoo_client.c` to sources.
- [ ] **Step 4: Build** — expected `.uf2` builds (odoo_client not yet called from main; that's Task 19).
- [ ] **Step 5: Commit** — `feat: odoo sync orchestration with automatic re-authentication`.

---

### Task 19: Final superloop `main.c`

**Files:**
- Create: `include/app.h`
- Modify: `src/main.c` (full replacement)

**Interfaces:**
- Consumes: everything above.
- Produces: the shipping firmware. Boot: stdio → watchdog(8 s) → wifi init (fatal-blink on failure) → epd_io_init → wifi connect w/ backoff → SNTP wait → superloop: sync → note result (3 fails → offline snapshot) → `refresh_needed` → render+display+sleep → idle `POLL_INTERVAL_S`.

- [ ] **Step 1: Create `include/app.h`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Top-level application state owned by main
*/

#ifndef APP_H_
    #define APP_H_

    #include <stdint.h>
    #include "gfx.h"
    #include "refresh.h"

struct app {
    uint8_t fb[GFX_BUFFER_SIZE];
    struct snapshot displayed;
    struct snapshot current;
    int uid;
    unsigned int fails;
    unsigned int has_displayed;
    uint32_t last_refresh_s;
    char offline_since[8];
};

#endif /* !APP_H_ */
```

- [ ] **Step 2: Replace `src/main.c`**

```c
/*
** EPITECH PROJECT, 2026
** epaper_dashboard
** File description:
** Boot sequence and superloop: poll odoo, refresh the panel on change
*/

#include <stdio.h>
#include "pico/stdlib.h"
#include "pico/cyw43_arch.h"
#include "hardware/watchdog.h"
#include "app.h"
#include "config.h"
#include "dashboard.h"
#include "epd.h"
#include "net_time.h"
#include "net_wifi.h"
#include "odoo_client.h"
#include "sys_idle.h"
#include "time_fmt.h"

static uint32_t now_s(void)
{
    return (uint32_t)(to_ms_since_boot(get_absolute_time()) / 1000u);
}

static void fatal_blink(void)
{
    for (;;) {
        net_wifi_led(1);
        sleep_ms(100);
        net_wifi_led(0);
        sleep_ms(100);
    }
}

static void wifi_boot(void)
{
    uint32_t backoff = 5000;

    while (net_wifi_connect() != 0) {
        printf("wifi: retry in %u ms\n", backoff);
        sys_idle_ms(backoff);
        backoff = backoff < 80000 ? backoff * 2 : 80000;
    }
    printf("wifi: connected\n");
}

static void time_boot(void)
{
    net_time_init();
    while (net_time_synced() == 0)
        sys_idle_ms(500);
    printf("time: synced\n");
}

static void note_result(struct app *app, int ret)
{
    struct tm lt;

    if (ret == 0) {
        app->fails = 0;
        app->current.offline = 0;
        printf("odoo: %u tasks\n", app->current.list.count);
        return;
    }
    app->fails++;
    app->current.list = app->displayed.list;
    printf("odoo: fetch failed (%u in a row)\n", app->fails);
    if (app->fails == 3) {
        app->current.offline = 1;
        net_time_local(&lt);
        time_fmt_hhmm(app->offline_since, sizeof(app->offline_since),
            &lt);
    }
}

static int display_frame(struct app *app)
{
    int ret = 0;

    printf("epd: refresh start\n");
    if (epd_init() != 0) {
        printf("epd: panel not responding\n");
        return -1;
    }
    ret = epd_display(app->fb);
    epd_sleep();
    printf("epd: refresh %s\n", ret == 0 ? "done" : "failed");
    return ret;
}

static void apply_refresh(struct app *app)
{
    struct dashboard_data data;
    struct tm lt;

    data.snap = &app->current;
    net_time_local(&lt);
    data.today = lt;
    time_fmt_banner(data.banner_date, sizeof(data.banner_date), &lt);
    time_fmt_hhmm(data.updated_hhmm, sizeof(data.updated_hhmm), &lt);
    snprintf(data.offline_since, sizeof(data.offline_since), "%s",
        app->offline_since);
    dashboard_render(app->fb, &data);
    if (display_frame(app) != 0)
        return;
    app->displayed = app->current;
    app->last_refresh_s = now_s();
    app->has_displayed = 1;
}

static void poll_once(struct app *app)
{
    struct refresh_times t = {now_s(), app->last_refresh_s,
        app->has_displayed};
    int ret = 0;

    if (net_wifi_up() == 0)
        net_wifi_connect();
    ret = odoo_client_sync(&app->uid, &app->current.list);
    note_result(app, ret);
    if (ret != 0 && app->fails < 3)
        return;
    if (refresh_needed(&app->displayed, &app->current, &t) != 0)
        apply_refresh(app);
}

int main(void)
{
    static struct app app;

    stdio_init_all();
    watchdog_enable(8000, 1);
    if (net_wifi_init() != 0)
        fatal_blink();
    epd_io_init();
    wifi_boot();
    time_boot();
    for (;;) {
        poll_once(&app);
        sys_idle_ms((uint32_t)POLL_INTERVAL_S * 1000u);
    }
    return 0;
}
```

(`main.c` has 9 functions total, 1 exported — within C-O3. `refresh_times.now_s` is captured before the sync; the ~seconds of drift is irrelevant at 180 s/86400 s thresholds.)

- [ ] **Step 3: Build** — expected `.uf2` builds cleanly.
- [ ] **Step 4 [HW] End-to-end validation with the user** (real `include/config.h` filled by the user first). Staged serial checklist from the spec §12:
  1. `wifi: connected` 2. `time: synced` 3. `odoo: authenticated uid=N` 4. `odoo: N tasks` 5. `epd: refresh start` → panel flashes ~30 s → `epd: refresh done` 6. next polls print `odoo: N tasks` with **no** `epd:` lines while nothing changed 7. change a task in Odoo → next poll refreshes 8. unplug router → 3 fails → offline footer; replug → recovery 9. `git status` shows `include/config.h` untracked.
- [ ] **Step 5: Commit** — `feat: superloop with refresh-on-change and offline handling`.

---

### Task 20: README

**Files:**
- Create: `README.md` (replacing any stub content)

**Interfaces:** Consumes everything; produces user documentation.

- [ ] **Step 1: Write `README.md`** covering, in this order (write real prose, base every command on what Tasks 1–19 actually did):
  1. What the project is (one paragraph + the dashboard layout sketch from the spec §8).
  2. Hardware + wiring table (copy from `DRIVER_REFERENCE.md` §1), SPI-Select switch on 0, panel safety rules (sleep after refresh, ≥3 min spacing, daily refresh).
  3. Toolchain setup from zero: the winget installs, pico-sdk clone + submodules, `PICO_SDK_PATH`/`PICO_TOOLCHAIN_PATH` env vars (Task 1 commands, as actually executed).
  4. Configuration: `cp config.h.example include/config.h`, field-by-field explanation; how to create an Odoo API key (Odoo → My Profile → Account Security → New API Key); `ODOO_TASK_DOMAIN` tuning (the `%d` = uid; what to do if the instance lacks `is_closed` — e.g. filter `["stage_id.fold","=",false]`); warning to keep `"` out of config values.
  5. Optional CA pinning: `openssl s_client -connect HOST:443 -showcerts`, take the last certificate block, convert to a single-string `#define` (mention the style exemption for the private config.h); note that without it TLS encrypts but doesn't authenticate the server.
  6. Build + flash: `cmake -S . -B build -G Ninja`, `cmake --build build`, BOOTSEL drag-drop of `build/epaper_dashboard.uf2`, serial monitor at 115200.
  7. Host tests: `make -C tests run`.
  8. Behavior reference: poll every 5 min, refresh only on change / daily; offline footer after 3 failed polls; watchdog reboot on hang; LED fast-blink = wifi chip init failed.
  9. Troubleshooting: panel unresponsive → DRIVER_REFERENCE.md §6; mirrored text → flip bit order in `gfx_text.c` (Task 10 note).
- [ ] **Step 2: Commit** — `docs: full setup, configuration and behavior reference`.

---

### Task 21: Style audit and fixes

**Files:**
- Create: `tests/check_style.py`
- Modify: any file with violations.

**Interfaces:** Consumes all `.c`/`.h` under `src/`, `include/`, `tests/`. Produces a passing audit.

- [ ] **Step 1: Create `tests/check_style.py`**

```python
#!/usr/bin/env python3
"""Heuristic Epitech C style audit for src/, include/ and tests/."""
import re
import sys
from pathlib import Path

HEADER_RE = re.compile(
    r"\A/\*\n\*\* EPITECH PROJECT, \d{4}\n\*\* .+\n"
    r"\*\* File description:\n(\*\* .+\n)+\*/\n")
FUNC_RE = re.compile(r"^[a-z]", re.M)

def file_errors(path):
    errors = []
    raw = path.read_bytes()
    if b"\r" in raw:
        errors.append("CRLF line ending")
    text = raw.decode("utf-8", errors="replace")
    if path.suffix in (".c", ".h") and not HEADER_RE.match(text):
        errors.append("missing or malformed EPITECH header")
    for i, line in enumerate(text.split("\n"), 1):
        if len(line) > 80:
            errors.append(f"line {i}: over 80 columns")
        if line != line.rstrip():
            errors.append(f"line {i}: trailing whitespace")
        if "\t" in line and path.suffix in (".c", ".h"):
            errors.append(f"line {i}: tab character")
    if path.suffix == ".c":
        errors += function_errors(text)
    return errors

def function_errors(text):
    errors = []
    lines = text.split("\n")
    exported = total = 0
    i = 0
    while i < len(lines):
        if lines[i] == "{" and i > 0 and "(" in lines[i - 1] \
                and not lines[i - 1].startswith(" "):
            total += 1
            sig = " ".join(lines[max(0, i - 3):i])
            if "static" not in sig.split("(")[0]:
                exported += 1
            depth, body = 1, 0
            j = i + 1
            while j < len(lines) and depth > 0:
                depth += lines[j].count("{") - lines[j].count("}")
                body += 1
                j += 1
            if body - 1 > 20:
                errors.append(f"function ending line {j}: body > 20 lines")
            i = j
        else:
            i += 1
    if exported > 5:
        errors.append(f"{exported} exported functions (max 5)")
    if total > 10:
        errors.append(f"{total} total functions (max 10)")
    return errors

def main():
    bad = 0
    for folder in ("src", "include", "tests"):
        for path in sorted(Path(folder).glob("**/*")):
            if path.suffix not in (".c", ".h") \
                    or "test_config" in str(path):
                pass
            if path.suffix not in (".c", ".h"):
                continue
            errors = file_errors(path)
            for err in errors:
                print(f"{path}: {err}")
            bad += len(errors)
    print("style: OK" if bad == 0 else f"style: {bad} issue(s)")
    return 1 if bad else 0

if __name__ == "__main__":
    sys.exit(main())
```

- [ ] **Step 2: Run** `python tests/check_style.py` from the repo root. Fix every reported issue in project files (the checker is heuristic — verify each hit by eye; false positives on multiline signatures are possible, adjust the code, not the rule).
- [ ] **Step 3: Manual pass over rules the script can't check**, file by file in `src/` and `include/`: declarations at top + blank line (C-L5/L6), no comments inside functions (C-F8), ≤4 params (C-F5), nesting ≤2 (C-C1), one empty line between functions (C-G2), `const` correctness (C-A1), pointer asterisk placement (C-V3), include guards + directive indentation (C-H2/C-G3).
- [ ] **Step 4: Re-run tests and build** — `make -C tests run` all OK, `cmake --build build` clean.
- [ ] **Step 5: Commit** — `chore: style audit tooling and compliance fixes`.

---

## Plan self-review (done at authoring time)

- **Spec coverage:** §1–2 hardware/constraints → Tasks 14 (driver, pacing, safety) and Global Constraints; §3 architecture/boot/loop → Task 19; §4 layout/modules → Tasks 2–19 (extra files listed as deviation 4); §5 wifi/sntp/tls → Tasks 15–17; §6 protocol → Tasks 6–7; §7 http → Tasks 11, 17; §8 rendering → Tasks 9, 10, 13; §9 refresh/failure policy → Tasks 12, 19; §10 config → Tasks 2, 4 (deviation 3); §11 memory → static buffers throughout; §12 testing → per-task tests + Task 19 Step 4 checklist; §13 build → Tasks 1, 3, README (CLI toolchain instead of VS Code extension — same SDK, scriptable; README documents both entry points); §14 out-of-scope respected.
- **Placeholder scan:** all code blocks complete; the only "adapt" instructions are environment discovery (installer paths, SDK tag) which cannot be pinned in advance — each has concrete commands and a verification step.
- **Type consistency:** `struct snapshot`/`refresh_times` (T12) match usage in T13/T19; `jsmn_ctx` (T6) matches T6 usage; `http_ctx` field order in T17 initializer matches its declaration; `gfx_style`/`gfx_rect` (T9) match T10/T13; `odoo_task_list` (T6) matches T7/T12/T13/T18/T19.
