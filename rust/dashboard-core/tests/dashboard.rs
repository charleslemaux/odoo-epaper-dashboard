use dashboard_core::dashboard::{self, DashboardData};
use dashboard_core::gfx::{Color, Frame, Rect, BUFFER_SIZE, WIDTH};
use dashboard_core::refresh::Snapshot;
use dashboard_core::time_fmt::Date;

fn px(fb: &Frame, x: i32, y: i32) -> u8 {
    let idx = (y as usize * WIDTH as usize + x as usize) / 2;
    if x & 1 == 0 {
        fb[idx] >> 4
    } else {
        fb[idx] & 0x0f
    }
}

fn region_has(fb: &Frame, r: &Rect, c: Color) -> bool {
    for y in r.y..r.y + r.h {
        for x in r.x..r.x + r.w {
            if px(fb, x, y) == c as u8 {
                return true;
            }
        }
    }
    false
}

fn data<'a>(snap: &'a Snapshot) -> DashboardData<'a> {
    DashboardData {
        snap,
        today: Date {
            year: 2026,
            month: 8,
            day: 26,
        },
        banner_date: "mer 26/08",
        updated_hhmm: "14:35",
        offline_since: "",
    }
}

fn new_frame() -> Box<Frame> {
    Box::new([0u8; BUFFER_SIZE])
}

#[test]
fn banner_and_rows() {
    let mut snap = Snapshot::default();
    snap.list.count = 1;
    snap.list.total = 1;
    let _ = snap.list.items[0].name.push_str("Activite urgente");
    let _ = snap.list.items[0].deadline.push_str("2026-08-20");
    let _ = snap.list.items[0].icon.push_str("fa-phone");
    let mut fb = new_frame();
    dashboard::render(&mut fb, &data(&snap));
    assert_eq!(px(&fb, 5, 5), Color::Green as u8);
    assert!(region_has(
        &fb,
        &Rect {
            x: 16,
            y: 80,
            w: 300,
            h: 48
        },
        Color::Black
    ));
    assert!(region_has(
        &fb,
        &Rect {
            x: WIDTH - 160,
            y: 80,
            w: 160,
            h: 48
        },
        Color::Red
    ));
}

#[test]
fn empty_state() {
    let snap = Snapshot::default();
    let mut fb = new_frame();
    dashboard::render(&mut fb, &data(&snap));
    assert!(region_has(
        &fb,
        &Rect {
            x: 0,
            y: 200,
            w: WIDTH,
            h: 120
        },
        Color::Green
    ));
}

#[test]
fn offline_footer() {
    let snap = Snapshot {
        offline: true,
        ..Default::default()
    };
    let mut fb = new_frame();
    let mut d = data(&snap);
    d.offline_since = "14:00";
    dashboard::render(&mut fb, &d);
    assert!(region_has(
        &fb,
        &Rect {
            x: 0,
            y: 440,
            w: WIDTH,
            h: 40
        },
        Color::Red
    ));
}

#[test]
fn overflow_footer() {
    let mut snap = Snapshot::default();
    snap.list.count = 1;
    snap.list.total = 4;
    let _ = snap.list.items[0].name.push_str("Une activite");
    let mut fb = new_frame();
    dashboard::render(&mut fb, &data(&snap));
    assert!(region_has(
        &fb,
        &Rect {
            x: 0,
            y: 440,
            w: 400,
            h: 40
        },
        Color::Black
    ));
}

#[test]
fn deterministic() {
    let mut snap = Snapshot::default();
    snap.list.count = 1;
    snap.list.total = 1;
    let _ = snap.list.items[0].name.push_str("Stable");
    let mut a = new_frame();
    let mut b = new_frame();
    dashboard::render(&mut a, &data(&snap));
    dashboard::render(&mut b, &data(&snap));
    assert_eq!(a[..], b[..]);
}
