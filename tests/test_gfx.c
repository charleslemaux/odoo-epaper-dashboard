/*
** Charles Le Maux, 2026
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
