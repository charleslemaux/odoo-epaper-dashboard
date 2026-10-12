use dashboard_core::gfx::{self, Color, Frame, Rect, BUFFER_SIZE, HEIGHT, WIDTH};

fn px(fb: &Frame, x: i32, y: i32) -> u8 {
    let idx = (y as usize * WIDTH as usize + x as usize) / 2;
    if x & 1 == 0 {
        fb[idx] >> 4
    } else {
        fb[idx] & 0x0f
    }
}

fn new_frame() -> Box<Frame> {
    Box::new([0u8; BUFFER_SIZE])
}

#[test]
fn fill_and_pixel() {
    let mut fb = new_frame();
    gfx::fill(&mut fb, Color::White);
    assert_eq!(fb[0], 0x11);
    gfx::pixel(&mut fb, 0, 0, Color::Black);
    assert_eq!(fb[0], 0x01);
    gfx::pixel(&mut fb, 1, 0, Color::Red);
    assert_eq!(fb[0], 0x03);
    assert_eq!(px(&fb, 2, 0), Color::White as u8);
}

#[test]
fn clipping() {
    let mut fb = new_frame();
    gfx::fill(&mut fb, Color::White);
    gfx::pixel(&mut fb, -1, 0, Color::Black);
    gfx::pixel(&mut fb, WIDTH, 0, Color::Black);
    gfx::pixel(&mut fb, 0, HEIGHT, Color::Black);
    gfx::fill_rect(
        &mut fb,
        &Rect {
            x: -10,
            y: -10,
            w: 20,
            h: 20,
        },
        Color::Green,
    );
    assert_eq!(px(&fb, 0, 0), Color::Green as u8);
    assert_eq!(px(&fb, 9, 9), Color::Green as u8);
    assert_eq!(px(&fb, 10, 10), Color::White as u8);
}

#[test]
fn rect_outline() {
    let mut fb = new_frame();
    gfx::fill(&mut fb, Color::White);
    gfx::rect(
        &mut fb,
        &Rect {
            x: 10,
            y: 10,
            w: 5,
            h: 5,
        },
        Color::Blue,
    );
    assert_eq!(px(&fb, 10, 10), Color::Blue as u8);
    assert_eq!(px(&fb, 14, 14), Color::Blue as u8);
    assert_eq!(px(&fb, 12, 12), Color::White as u8);
}
