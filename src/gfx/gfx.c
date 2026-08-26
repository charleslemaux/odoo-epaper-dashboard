/*
** Charles Le Maux, 2026
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
