/*
** Charles Le Maux, 2026
** epaper_dashboard
** File description:
** 4bpp GS4_HMSB framebuffer graphics primitives and text rendering
*/

#ifndef GFX_H_
    #define GFX_H_

    #include <stddef.h>
    #include <stdint.h>

#define GFX_WIDTH 800
#define GFX_HEIGHT 480
#define GFX_BUFFER_SIZE (GFX_WIDTH * GFX_HEIGHT / 2)

enum gfx_color {
    GFX_BLACK = 0x0,
    GFX_WHITE = 0x1,
    GFX_YELLOW = 0x2,
    GFX_RED = 0x3,
    GFX_BLUE = 0x5,
    GFX_GREEN = 0x6,
};

struct gfx_rect {
    int x;
    int y;
    int w;
    int h;
};

struct gfx_style {
    int x;
    int y;
    int color;
    int scale;
};

void gfx_fill(uint8_t *fb, int color);
void gfx_pixel(uint8_t *fb, int x, int y, int color);
void gfx_fill_rect(uint8_t *fb, struct gfx_rect const *r, int color);
void gfx_rect(uint8_t *fb, struct gfx_rect const *r, int color);
void gfx_text(uint8_t *fb, struct gfx_style const *st, char const *s);
void gfx_text_centered(uint8_t *fb, struct gfx_style const *st,
    char const *s);
int gfx_text_width(char const *s, int scale);
int gfx_text_fit(char const *s, int scale, int max_width);

#endif /* !GFX_H_ */
