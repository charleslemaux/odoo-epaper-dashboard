/*
** Charles Le Maux, 2026
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
