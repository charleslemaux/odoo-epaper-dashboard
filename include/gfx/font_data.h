/*
** Charles Le Maux, 2026
** epaper_dashboard
** File description:
** Generated bitmap font data (Questrial), see tools/gen_font.py
*/

#ifndef FONT_DATA_H_
    #define FONT_DATA_H_

    #include <stdint.h>

struct gfx_glyph {
    uint16_t offset;
    uint8_t width;
    uint8_t advance;
};

struct gfx_font {
    int height;
    struct gfx_glyph const *glyphs;
    uint8_t const *bitmap;
};

extern const struct gfx_font GFX_FONT_32;
extern const struct gfx_font GFX_FONT_48;

#endif /* !FONT_DATA_H_ */
