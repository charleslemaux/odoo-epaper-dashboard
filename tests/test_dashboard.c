/*
** Charles Le Maux, 2026
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
    struct gfx_rect name = {16, 80, 300, 48};
    struct gfx_rect date = {GFX_WIDTH - 160, 80, 160, 48};

    fill_data(&d, &snap);
    snap.list.count = 1;
    snprintf(snap.list.items[0].name, 64, "Activite urgente");
    snprintf(snap.list.items[0].deadline, 11, "2026-08-20");
    snprintf(snap.list.items[0].icon, 24, "fa-phone");
    dashboard_render(fb, &d);
    assert(px(fb, 5, 5) == GFX_GREEN);
    assert(region_has(fb, &name, GFX_BLACK));
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
    struct gfx_rect footer = {0, 440, GFX_WIDTH, 40};

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
    snprintf(snap.list.items[0].name, 64, "Stable");
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
