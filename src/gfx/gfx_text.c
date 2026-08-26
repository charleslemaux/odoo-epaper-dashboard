/*
** Charles Le Maux, 2026
** epaper_dashboard
** File description:
** Proportional bitmap text rendering using the embedded Questrial font
*/

#include "gfx.h"
#include "font_data.h"

static struct gfx_font const *font_for(int scale)
{
    if (scale >= 3)
        return &GFX_FONT_24;
    return &GFX_FONT_16;
}

static struct gfx_glyph const *glyph_for(struct gfx_font const *font,
    char c)
{
    unsigned char code = (unsigned char)c;

    if (code < 0x20 || code > 0x7E)
        code = '?';
    return &font->glyphs[code - 0x20];
}

static void draw_row(uint8_t *fb, struct gfx_style const *pos,
    uint8_t const *row, int width)
{
    for (int col = 0; col < width; col++) {
        if ((row[col / 8] >> (7 - (col % 8))) & 1)
            gfx_pixel(fb, pos->x + col, pos->y, pos->color);
    }
}

static void draw_glyph(uint8_t *fb, struct gfx_style const *st,
    struct gfx_font const *font, struct gfx_glyph const *glyph)
{
    struct gfx_style pos = *st;
    int stride = (glyph->width + 7) / 8;
    uint8_t const *rows = &font->bitmap[glyph->offset];

    for (int row = 0; row < font->height; row++) {
        draw_row(fb, &pos, &rows[(size_t)row * (size_t)stride],
            glyph->width);
        pos.y++;
    }
}

void gfx_text(uint8_t *fb, struct gfx_style const *st, char const *s)
{
    struct gfx_font const *font = font_for(st->scale);
    struct gfx_style cur = *st;

    for (size_t i = 0; s[i] != '\0'; i++) {
        draw_glyph(fb, &cur, font, glyph_for(font, s[i]));
        cur.x += glyph_for(font, s[i])->advance;
    }
}

int gfx_text_width(char const *s, int scale)
{
    struct gfx_font const *font = font_for(scale);
    int width = 0;

    for (size_t i = 0; s[i] != '\0'; i++)
        width += glyph_for(font, s[i])->advance;
    return width;
}

void gfx_text_centered(uint8_t *fb, struct gfx_style const *st,
    char const *s)
{
    struct gfx_style cur = *st;

    cur.x = (GFX_WIDTH - gfx_text_width(s, st->scale)) / 2;
    gfx_text(fb, &cur, s);
}
