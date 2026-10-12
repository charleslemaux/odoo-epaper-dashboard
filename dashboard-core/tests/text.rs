use dashboard_core::gfx::{self, Color, Frame, Rect, Style, BUFFER_SIZE, WIDTH};
use dashboard_core::icon_data::Icon;
use dashboard_core::text;

fn px(fb: &Frame, x: i32, y: i32) -> u8 {
    let idx = (y as usize * WIDTH as usize + x as usize) / 2;
    if x & 1 == 0 {
        fb[idx] >> 4
    } else {
        fb[idx] & 0x0f
    }
}

fn count_color(fb: &Frame, r: &Rect, c: Color) -> u32 {
    let mut n = 0;
    for y in r.y..r.y + r.h {
        for x in r.x..r.x + r.w {
            if px(fb, x, y) == c as u8 {
                n += 1;
            }
        }
    }
    n
}

fn white_frame() -> Box<Frame> {
    let mut fb = Box::new([0u8; BUFFER_SIZE]);
    gfx::fill(&mut fb, Color::White);
    fb
}

#[test]
fn glyph_draws_pixels() {
    let mut fb = white_frame();
    text::text(
        &mut fb,
        &Style {
            x: 0,
            y: 0,
            color: Color::Black,
            scale: 2,
        },
        "A",
    );
    assert!(
        count_color(
            &fb,
            &Rect {
                x: 0,
                y: 0,
                w: 64,
                h: 32
            },
            Color::Black
        ) > 8
    );
    assert_eq!(
        count_color(
            &fb,
            &Rect {
                x: 0,
                y: 32,
                w: WIDTH,
                h: 32
            },
            Color::Black
        ),
        0
    );
}

#[test]
fn large_font_is_taller() {
    let mut fb = white_frame();
    text::text(
        &mut fb,
        &Style {
            x: 0,
            y: 0,
            color: Color::Black,
            scale: 3,
        },
        "Ag",
    );
    assert!(
        count_color(
            &fb,
            &Rect {
                x: 0,
                y: 32,
                w: WIDTH,
                h: 16
            },
            Color::Black
        ) > 0
    );
}

#[test]
fn proportional_widths() {
    let wide = text::text_width("WWW", 2);
    let narrow = text::text_width("iii", 2);
    assert!(wide > 0 && narrow > 0 && wide > narrow);
    assert!(text::text_width("WWW", 3) > wide);
}

#[test]
fn centering() {
    let mut fb = white_frame();
    let w = text::text_width("ab", 2);
    text::text_centered(
        &mut fb,
        &Style {
            x: 0,
            y: 100,
            color: Color::Red,
            scale: 2,
        },
        "ab",
    );
    assert_eq!(
        count_color(
            &fb,
            &Rect {
                x: 0,
                y: 100,
                w: (WIDTH - w) / 2 - 1,
                h: 32
            },
            Color::Red
        ),
        0
    );
    assert!(
        count_color(
            &fb,
            &Rect {
                x: 0,
                y: 100,
                w: WIDTH,
                h: 32
            },
            Color::Red
        ) > 0
    );
}

#[test]
fn icon_draws_pixels() {
    let mut fb = white_frame();
    text::icon(
        &mut fb,
        &Style {
            x: 0,
            y: 0,
            color: Color::Black,
            scale: 2,
        },
        Icon::Envelope,
    );
    assert!(
        count_color(
            &fb,
            &Rect {
                x: 0,
                y: 0,
                w: 32,
                h: 32
            },
            Color::Black
        ) > 16
    );
    assert_eq!(
        count_color(
            &fb,
            &Rect {
                x: 32,
                y: 0,
                w: WIDTH - 32,
                h: 32
            },
            Color::Black
        ),
        0
    );
}

#[test]
fn text_fit_is_pixel_accurate() {
    let full = text::text_width("Hello world", 2);
    assert_eq!(text::text_fit("Hello world", 2, full), 11);
    let fit = text::text_fit("Hello world", 2, full - 1);
    assert!(fit < 11);
    assert!(text::text_width(&"Hello world"[..fit], 2) < full);
}

#[test]
fn deterministic() {
    let mut a = white_frame();
    let mut b = white_frame();
    let st = Style {
        x: 10,
        y: 10,
        color: Color::Black,
        scale: 2,
    };
    text::text(&mut a, &st, "Stable text 123");
    text::text(&mut b, &st, "Stable text 123");
    assert_eq!(a[..], b[..]);
}
