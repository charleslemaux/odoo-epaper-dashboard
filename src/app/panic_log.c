/*
** Charles Le Maux, 2026
** epaper_dashboard
** File description:
** Panic message capture surviving reboot for headless diagnostics
*/

#include <stdarg.h>
#include <stdio.h>
#include "pico/stdlib.h"
#include "pico/platform/sections.h"
#include "hardware/watchdog.h"
#include "panic_log.h"

static struct panic_log *get_log(void)
{
    static struct panic_log __uninitialized_ram(log);

    return &log;
}

void panic_log_store(char const *fmt, ...)
{
    struct panic_log *log = get_log();
    va_list args;

    va_start(args, fmt);
    vsnprintf(log->message, sizeof(log->message),
        fmt != 0 ? fmt : "(no message)", args);
    va_end(args);
    log->magic = PANIC_LOG_MAGIC;
    watchdog_reboot(0, 0, 200);
    for (;;)
        tight_loop_contents();
}

void panic_log_report(void)
{
    struct panic_log *log = get_log();

    if (log->magic != PANIC_LOG_MAGIC)
        return;
    printf("last panic: %s\n", log->message);
    log->magic = 0;
}
