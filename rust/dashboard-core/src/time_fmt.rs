//! Date formatting and deadline classification (port of src/app/time_fmt.c,
//! with Hinnant civil-date arithmetic replacing mktime/gmtime).

use core::fmt::Write;
use heapless::String;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Date {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DeadlineClass {
    None,
    Overdue,
    Soon,
    Normal,
}

const DAY_NAMES: [&str; 7] = ["dim", "lun", "mar", "mer", "jeu", "ven", "sam"];

pub fn days_from_civil(d: Date) -> i64 {
    let y = i64::from(d.year) - i64::from(d.month <= 2);
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (i64::from(d.month) + 9) % 12;
    let doy = (153 * mp + 2) / 5 + i64::from(d.day) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

pub fn civil_from_days(z: i64) -> Date {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u8;
    let month = (if mp < 10 { mp + 3 } else { mp - 9 }) as u8;
    Date {
        year: (y + i64::from(month <= 2)) as i32,
        month,
        day,
    }
}

pub fn weekday(d: Date) -> u8 {
    (days_from_civil(d) + 4).rem_euclid(7) as u8
}

pub fn banner(d: Date) -> String<16> {
    let mut s = String::new();
    let _ = write!(
        s,
        "{} {:02}/{:02}",
        DAY_NAMES[weekday(d) as usize],
        d.day,
        d.month
    );
    s
}

pub fn hhmm(hour: u8, minute: u8) -> String<8> {
    let mut s = String::new();
    let _ = write!(s, "{hour:02}:{minute:02}");
    s
}

pub fn ddmm(iso: &str) -> String<8> {
    let mut s = String::new();
    if iso.len() < 10 {
        let _ = s.push_str("--");
        return s;
    }
    let b = iso.as_bytes();
    let _ = write!(
        s,
        "{}{}/{}{}",
        b[8] as char, b[9] as char, b[5] as char, b[6] as char
    );
    s
}

fn parse_iso(iso: &str) -> Option<Date> {
    if iso.len() < 10 || iso.as_bytes()[4] != b'-' || iso.as_bytes()[7] != b'-' {
        return None;
    }
    Some(Date {
        year: iso[0..4].parse().ok()?,
        month: iso[5..7].parse().ok()?,
        day: iso[8..10].parse().ok()?,
    })
}

pub fn days_late(iso: &str, today: Date) -> i64 {
    match parse_iso(iso) {
        Some(due) => days_from_civil(today) - days_from_civil(due),
        None => 0,
    }
}

pub fn deadline_class(iso: &str, today: Date) -> DeadlineClass {
    if parse_iso(iso).is_none() {
        return DeadlineClass::None;
    }
    match days_late(iso, today) {
        late if late > 0 => DeadlineClass::Overdue,
        0 | -1 => DeadlineClass::Soon,
        _ => DeadlineClass::Normal,
    }
}
