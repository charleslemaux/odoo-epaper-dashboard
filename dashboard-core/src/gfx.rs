//! 4bpp `GS4_HMSB` framebuffer primitives (port of `src/gfx/gfx.c`).

pub const WIDTH: i32 = 800;
pub const HEIGHT: i32 = 480;
pub const BUFFER_SIZE: usize = (WIDTH as usize) * (HEIGHT as usize) / 2;
pub type Frame = [u8; BUFFER_SIZE];

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Color {
    Black = 0x0,
    White = 0x1,
    Yellow = 0x2,
    Red = 0x3,
    Blue = 0x5,
    Green = 0x6,
}

#[derive(Clone, Copy)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

#[derive(Clone, Copy)]
pub struct Style {
    pub x: i32,
    pub y: i32,
    pub color: Color,
    pub scale: u8,
}

pub fn fill(fb: &mut Frame, c: Color) {
    let b = ((c as u8) << 4) | (c as u8);
    fb.fill(b);
}

pub fn pixel(fb: &mut Frame, x: i32, y: i32, c: Color) {
    if x < 0 || y < 0 || x >= WIDTH || y >= HEIGHT {
        return;
    }
    // x, y are non-negative here (checked above); C-parity arithmetic
    // mirrors gfx.c's `(y * WIDTH + x) / 2` byte-offset computation.
    #[allow(clippy::cast_sign_loss)]
    let (x, y) = (x as usize, y as usize);
    let idx = usize::midpoint(y * WIDTH as usize, x);
    if x & 1 == 0 {
        fb[idx] = (fb[idx] & 0x0f) | ((c as u8) << 4);
    } else {
        fb[idx] = (fb[idx] & 0xf0) | (c as u8);
    }
}

/// Fills `r` with `c`, one packed-byte slice `fill` per full-width row span
/// (the separator/banner hot path) instead of a per-pixel loop. A row's
/// byte at index `b` packs columns `2*b` and `2*b + 1`, so a byte can only
/// be slice-filled when both its columns lie inside the (clipped) rect;
/// the up-to-one odd column at each edge still goes through `pixel`.
pub fn fill_rect(fb: &mut Frame, r: &Rect, c: Color) {
    let y0 = r.y.max(0);
    let y1 = (r.y + r.h).min(HEIGHT);
    let x0 = r.x.max(0);
    let x1 = (r.x + r.w).min(WIDTH);
    if y1 <= y0 || x1 <= x0 {
        return;
    }
    // Merged byte for a full pixel pair, hoisted out of the row loop
    // (same nibble packing as `fill`).
    let byte = ((c as u8) << 4) | (c as u8);
    // Round the clipped span inward to the nearest byte boundary: the
    // first/last column is only alone (needs `pixel`) when x0/x1 is odd.
    let aligned_x0 = (x0 + 1) & !1;
    let aligned_x1 = x1 & !1;
    for y in y0..y1 {
        // y, x0, x1, aligned_x0, aligned_x1 are all non-negative here
        // (clamped above); C-parity arithmetic, same byte-offset shape as
        // `pixel`'s `(y * WIDTH + x) / 2`.
        #[allow(clippy::cast_sign_loss)]
        let row_base = y as usize * (WIDTH as usize / 2);
        if x0 & 1 != 0 {
            pixel(fb, x0, y, c);
        }
        if aligned_x0 < aligned_x1 {
            #[allow(clippy::cast_sign_loss)]
            let (start, end) = (
                row_base + aligned_x0 as usize / 2,
                row_base + aligned_x1 as usize / 2,
            );
            fb[start..end].fill(byte);
        }
        if x1 & 1 != 0 {
            pixel(fb, x1 - 1, y, c);
        }
    }
}

pub fn rect(fb: &mut Frame, r: &Rect, c: Color) {
    fill_rect(
        fb,
        &Rect {
            x: r.x,
            y: r.y,
            w: r.w,
            h: 1,
        },
        c,
    );
    fill_rect(
        fb,
        &Rect {
            x: r.x,
            y: r.y + r.h - 1,
            w: r.w,
            h: 1,
        },
        c,
    );
    fill_rect(
        fb,
        &Rect {
            x: r.x,
            y: r.y,
            w: 1,
            h: r.h,
        },
        c,
    );
    fill_rect(
        fb,
        &Rect {
            x: r.x + r.w - 1,
            y: r.y,
            w: 1,
            h: r.h,
        },
        c,
    );
}
