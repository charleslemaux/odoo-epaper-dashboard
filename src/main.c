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
#include "panic_log.h"
#include "sys_idle.h"
#include "time_fmt.h"

static uint32_t now_s(void)
{
    return (uint32_t)(to_us_since_boot(get_absolute_time()) / 1000000ull);
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
        printf("wifi: retry in %u ms\n", (unsigned int)backoff);
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
    sleep_ms(2000);
    panic_log_report();
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
