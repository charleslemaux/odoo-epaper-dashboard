/*
** Charles Le Maux, 2026
** epaper_dashboard
** File description:
** Unit tests for proportional bitmap font text rendering
*/

#include <assert.h>
#include <stdio.h>
#include <string.h>
#include "gfx.h"
#include "font_data.h"

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
    struct gfx_style st = {0, 0, GFX_BLACK, 2};
    struct gfx_rect box = {0, 0, 64, 32};
    struct gfx_rect below = {0, 32, GFX_WIDTH, 32};

    gfx_fill(fb, GFX_WHITE);
    gfx_text(fb, &st, "A");
    assert(count_color(fb, &box, GFX_BLACK) > 8);
    assert(count_color(fb, &below, GFX_BLACK) == 0);
}

static void test_large_font_is_taller(void)
{
    static uint8_t fb[GFX_BUFFER_SIZE];
    struct gfx_style st = {0, 0, GFX_BLACK, 3};
    struct gfx_rect lower = {0, 32, GFX_WIDTH, 16};

    gfx_fill(fb, GFX_WHITE);
    gfx_text(fb, &st, "Ag");
    assert(count_color(fb, &lower, GFX_BLACK) > 0);
}

static void test_proportional_widths(void)
{
    int wide = gfx_text_width("WWW", 2);
    int narrow = gfx_text_width("iii", 2);

    assert(wide > 0);
    assert(narrow > 0);
    assert(wide > narrow);
    assert(gfx_text_width("WWW", 3) > wide);
}

static void test_centering(void)
{
    static uint8_t fb[GFX_BUFFER_SIZE];
    struct gfx_style st = {0, 100, GFX_RED, 2};
    int width = gfx_text_width("ab", 2);
    struct gfx_rect left = {0, 100, (GFX_WIDTH - width) / 2 - 1, 32};
    struct gfx_rect band = {0, 100, GFX_WIDTH, 32};

    gfx_fill(fb, GFX_WHITE);
    gfx_text_centered(fb, &st, "ab");
    assert(count_color(fb, &left, GFX_RED) == 0);
    assert(count_color(fb, &band, GFX_RED) > 0);
}

static void test_deterministic(void)
{
    static uint8_t fb1[GFX_BUFFER_SIZE];
    static uint8_t fb2[GFX_BUFFER_SIZE];
    struct gfx_style st = {10, 10, GFX_BLACK, 2};

    gfx_fill(fb1, GFX_WHITE);
    gfx_fill(fb2, GFX_WHITE);
    gfx_text(fb1, &st, "Stable text 123");
    gfx_text(fb2, &st, "Stable text 123");
    assert(memcmp(fb1, fb2, GFX_BUFFER_SIZE) == 0);
}

int main(void)
{
    test_glyph_draws_pixels();
    test_large_font_is_taller();
    test_proportional_widths();
    test_centering();
    test_deterministic();
    printf("test_gfx_text: OK\n");
    return 0;
}
