//! Dashboard layout rendering (port of src/app/dashboard.c).

use crate::activity_icon::icon_for;
use crate::gfx::{self, Color, Frame, Rect, Style, WIDTH};
use crate::odoo::Activity;
use crate::refresh::Snapshot;
use crate::text;
use crate::time_fmt::{self, Date, DeadlineClass};
use core::fmt::Write;
use heapless::String;

pub const BANNER_H: i32 = 72;
pub const ROWS_TOP: i32 = 80;
pub const ROW_H: i32 = 48;
pub const FOOTER_Y: i32 = 444;
pub const MARGIN: i32 = 16;
pub const GAP: i32 = 24;
pub const TEXT_X: i32 = 60;

pub struct DashboardData<'a> {
    pub snap: &'a Snapshot,
    pub today: Date,
    pub banner_date: &'a str,
    pub updated_hhmm: &'a str,
    pub offline_since: &'a str,
}

fn draw_banner(fb: &mut Frame, d: &DashboardData) {
    gfx::fill_rect(
        fb,
        &Rect {
            x: 0,
            y: 0,
            w: WIDTH,
            h: BANNER_H,
        },
        Color::Green,
    );
    let title = Style {
        x: MARGIN,
        y: 12,
        color: Color::White,
        scale: 3,
    };
    text::text(fb, &title, "Planning");
    let date_x = MARGIN + text::text_width("Planning", 3) + 40;
    text::text(
        fb,
        &Style {
            x: date_x,
            y: 24,
            color: Color::White,
            scale: 2,
        },
        d.banner_date,
    );
    let mut counter: String<24> = String::new();
    let _ = write!(counter, "{} activites", d.snap.list.total);
    let cx = WIDTH - MARGIN - text::text_width(&counter, 2);
    text::text(
        fb,
        &Style {
            x: cx,
            y: 24,
            color: Color::White,
            scale: 2,
        },
        &counter,
    );
}

fn deadline_color(act: &Activity, today: Date) -> Color {
    if time_fmt::deadline_class(&act.deadline, today) == DeadlineClass::Overdue {
        Color::Red
    } else {
        Color::Black
    }
}

fn draw_name(fb: &mut Frame, name: &str, y: i32, max_w: i32) -> Option<i32> {
    let st = Style {
        x: TEXT_X,
        y,
        color: Color::Black,
        scale: 2,
    };
    let fit = text::text_fit(name, 2, max_w);
    if fit == name.len() {
        text::text(fb, &st, name);
        return Some(st.x + text::text_width(name, 2));
    }
    let fit = text::text_fit(name, 2, max_w - text::text_width("...", 2));
    let mut line: String<72> = String::new();
    let _ = line.push_str(&name[..fit]);
    let _ = line.push_str("...");
    text::text(fb, &st, &line);
    None
}

fn draw_late(fb: &mut Frame, act: &Activity, today: Date, y: i32, to: i32) -> i32 {
    let days = time_fmt::days_late(&act.deadline, today);
    if days <= 0 {
        return to;
    }
    let mut chip: String<16> = String::new();
    let _ = write!(chip, "{days}j");
    let x = to - 12 - text::text_width(&chip, 2);
    text::text(
        fb,
        &Style {
            x,
            y,
            color: Color::Red,
            scale: 2,
        },
        &chip,
    );
    x
}

fn draw_extra(fb: &mut Frame, act: &Activity, y: i32, from: i32, to: i32) {
    let width = text::text_width(&act.record, 2);
    if act.record.is_empty() || width > to - from {
        return;
    }
    text::text(
        fb,
        &Style {
            x: to - width,
            y,
            color: Color::Black,
            scale: 2,
        },
        &act.record,
    );
}

fn draw_row(fb: &mut Frame, act: &Activity, y: i32, today: Date) {
    let ty = y + 8;
    text::icon(
        fb,
        &Style {
            x: MARGIN,
            y: ty,
            color: Color::Black,
            scale: 2,
        },
        icon_for(&act.icon),
    );
    let ddmm = time_fmt::ddmm(&act.deadline);
    let date_x = WIDTH - MARGIN - text::text_width(&ddmm, 2);
    text::text(
        fb,
        &Style {
            x: date_x,
            y: ty,
            color: deadline_color(act, today),
            scale: 2,
        },
        &ddmm,
    );
    let to = draw_late(fb, act, today, ty, date_x);
    let Some(name_end) = draw_name(fb, &act.name, ty, to - GAP - TEXT_X) else {
        return;
    };
    let from = name_end + GAP;
    let to = to - GAP;
    if to > from {
        draw_extra(fb, act, ty, from, to);
    }
}

fn draw_rows(fb: &mut Frame, d: &DashboardData) {
    let mut y = ROWS_TOP;
    for i in 0..d.snap.list.count as usize {
        draw_row(fb, &d.snap.list.items[i], y, d.today);
        gfx::fill_rect(
            fb,
            &Rect {
                x: 0,
                y: y + ROW_H - 2,
                w: WIDTH,
                h: 2,
            },
            Color::Black,
        );
        y += ROW_H;
    }
}

fn draw_footer(fb: &mut Frame, d: &DashboardData) {
    let mut line: String<48> = String::new();
    let _ = write!(line, "mise a jour {}", d.updated_hhmm);
    let rx = WIDTH - 8 - text::text_width(&line, 2);
    text::text(
        fb,
        &Style {
            x: rx,
            y: FOOTER_Y,
            color: Color::Black,
            scale: 2,
        },
        &line,
    );
    if d.snap.offline {
        let mut msg: String<48> = String::new();
        let _ = write!(msg, "HORS LIGNE depuis {}", d.offline_since);
        text::text(
            fb,
            &Style {
                x: 8,
                y: FOOTER_Y,
                color: Color::Red,
                scale: 2,
            },
            &msg,
        );
        return;
    }
    if d.snap.list.total > d.snap.list.count {
        let mut msg: String<48> = String::new();
        let _ = write!(
            msg,
            "+ d'autres activites ({})",
            d.snap.list.total - d.snap.list.count
        );
        text::text(
            fb,
            &Style {
                x: 8,
                y: FOOTER_Y,
                color: Color::Black,
                scale: 2,
            },
            &msg,
        );
    }
}

pub fn render(fb: &mut Frame, d: &DashboardData) {
    gfx::fill(fb, Color::White);
    draw_banner(fb, d);
    if d.snap.list.count == 0 {
        text::text_centered(
            fb,
            &Style {
                x: 0,
                y: 210,
                color: Color::Green,
                scale: 3,
            },
            "Aucune activite",
        );
    } else {
        draw_rows(fb, d);
    }
    draw_footer(fb, d);
}
