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
