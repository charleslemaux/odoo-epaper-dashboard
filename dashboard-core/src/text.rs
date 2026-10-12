//! Proportional bitmap text + icon rendering (port of `src/gfx/gfx_text.c`).

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
        usize::from(b - 0x20)
    } else {
        usize::from(b'?' - 0x20)
    }
}

// row (from `.enumerate()`) and col mirror gfx_text.c's `int` loop indices;
// the usize <-> i32 conversions below are C-parity arithmetic (both are
// always non-negative by construction: row is bounded by font.height
// (<=48), col by g.width).
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation
)]
fn draw_glyph(fb: &mut Frame, st: &Style, font: &Font, gi: usize) {
    let g = &font.glyphs[gi];
    let stride = usize::from(g.width).div_ceil(8);
    let width = i32::from(g.width);
    let color = st.color;
    let height = font.height as usize;
    let rows = &font.bitmap[usize::from(g.offset)..];
    for (row, bits) in rows.chunks(stride).take(height).enumerate() {
        let y = st.y + row as i32;
        for col in 0..width {
            if (bits[(col / 8) as usize] >> (7 - (col % 8))) & 1 == 1 {
                gfx::pixel(fb, st.x + col, y, color);
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
        cur.x += i32::from(font.glyphs[gi].advance);
    }
}

#[must_use]
pub fn text_width(s: &str, scale: u8) -> i32 {
    let font = font_for(scale);
    s.bytes()
        .map(|b| i32::from(font.glyphs[glyph_index(b)].advance))
        .sum()
}

#[must_use]
pub fn text_fit(s: &str, scale: u8, max_width: i32) -> usize {
    let font = font_for(scale);
    let mut width = 0;
    for (count, b) in s.bytes().enumerate() {
        width += i32::from(font.glyphs[glyph_index(b)].advance);
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
