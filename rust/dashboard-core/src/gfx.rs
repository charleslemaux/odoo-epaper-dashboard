//! 4bpp GS4_HMSB framebuffer primitives (port of src/gfx/gfx.c).

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
    let idx = (y as usize * WIDTH as usize + x as usize) / 2;
    if x & 1 == 0 {
        fb[idx] = (fb[idx] & 0x0f) | ((c as u8) << 4);
    } else {
        fb[idx] = (fb[idx] & 0xf0) | (c as u8);
    }
}

pub fn fill_rect(fb: &mut Frame, r: &Rect, c: Color) {
    for dy in 0..r.h {
        for dx in 0..r.w {
            pixel(fb, r.x + dx, r.y + dy, c);
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
