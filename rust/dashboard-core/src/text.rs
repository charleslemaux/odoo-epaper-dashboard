//! Proportional bitmap text + icon rendering (port of src/gfx/gfx_text.c).

use crate::font_data::{FONT_32, FONT_48};
use crate::fonts::Font;
use crate::gfx::{self, Frame, Style, WIDTH};
use crate::icon_data::{Icon, ICONS_32};

fn font_for(scale: u8) -> &'static Font {
    if scale >= 3 {
        &FONT_48
    } else {
        &FONT_32
    }
}

fn glyph_index(b: u8) -> usize {
    if (0x20..=0x7e).contains(&b) {
        (b - 0x20) as usize
    } else {
        (b'?' - 0x20) as usize
    }
}

fn draw_glyph(fb: &mut Frame, st: &Style, font: &Font, gi: usize) {
    let g = &font.glyphs[gi];
    let stride = (g.width as usize).div_ceil(8);
    let rows = &font.bitmap[g.offset as usize..];
    for row in 0..font.height {
        let r = &rows[row as usize * stride..];
        for col in 0..g.width as i32 {
            if (r[(col / 8) as usize] >> (7 - (col % 8))) & 1 == 1 {
                gfx::pixel(fb, st.x + col, st.y + row, st.color);
            }
        }
    }
}

pub fn text(fb: &mut Frame, st: &Style, s: &str) {
    let font = font_for(st.scale);
    let mut cur = *st;
    for &b in s.as_bytes() {
        let gi = glyph_index(b);
        draw_glyph(fb, &cur, font, gi);
        cur.x += font.glyphs[gi].advance as i32;
    }
}

pub fn text_width(s: &str, scale: u8) -> i32 {
    let font = font_for(scale);
    s.bytes()
        .map(|b| font.glyphs[glyph_index(b)].advance as i32)
        .sum()
}

pub fn text_fit(s: &str, scale: u8, max_width: i32) -> usize {
    let font = font_for(scale);
    let mut width = 0;
    for (count, b) in s.bytes().enumerate() {
        width += font.glyphs[glyph_index(b)].advance as i32;
        if width > max_width {
            return count;
        }
    }
    s.len()
}

pub fn text_centered(fb: &mut Frame, st: &Style, s: &str) {
    let mut cur = *st;
    cur.x = (WIDTH - text_width(s, st.scale)) / 2;
    text(fb, &cur, s);
}

pub fn icon(fb: &mut Frame, st: &Style, ic: Icon) {
    draw_glyph(fb, st, &ICONS_32, ic as usize);
}
