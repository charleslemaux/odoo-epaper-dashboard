use dashboard_core::time_fmt::{self, Date, DeadlineClass};

fn d(y: i32, m: u8, day: u8) -> Date {
    Date {
        year: y,
        month: m,
        day,
    }
}

#[test]
fn banner_and_hhmm() {
    assert_eq!(time_fmt::banner(d(2026, 8, 26)).as_str(), "mer 26/08");
    assert_eq!(time_fmt::hhmm(14, 5).as_str(), "14:05");
}

#[test]
fn ddmm() {
    assert_eq!(time_fmt::ddmm("2026-08-30").as_str(), "30/08");
    assert_eq!(time_fmt::ddmm("").as_str(), "--");
}

#[test]
fn deadline_classes() {
    let today = d(2026, 8, 26);
    assert_eq!(
        time_fmt::deadline_class("2026-08-25", today),
        DeadlineClass::Overdue
    );
    assert_eq!(
        time_fmt::deadline_class("2026-08-26", today),
        DeadlineClass::Soon
    );
    assert_eq!(
        time_fmt::deadline_class("2026-08-27", today),
        DeadlineClass::Soon
    );
    assert_eq!(
        time_fmt::deadline_class("2026-08-28", today),
        DeadlineClass::Normal
    );
    assert_eq!(time_fmt::deadline_class("", today), DeadlineClass::None);
}

#[test]
fn month_rollover() {
    let today = d(2026, 8, 31);
    assert_eq!(
        time_fmt::deadline_class("2026-09-01", today),
        DeadlineClass::Soon
    );
    assert_eq!(
        time_fmt::deadline_class("2026-09-02", today),
        DeadlineClass::Normal
    );
}

#[test]
fn days_late() {
    let today = d(2026, 8, 26);
    assert_eq!(time_fmt::days_late("2026-08-24", today), 2);
    assert_eq!(time_fmt::days_late("2026-08-26", today), 0);
    assert_eq!(time_fmt::days_late("2026-08-28", today), -2);
    assert_eq!(time_fmt::days_late("2026-07-26", today), 31);
    assert_eq!(time_fmt::days_late("", today), 0);
}

#[test]
fn civil_round_trip() {
    assert_eq!(time_fmt::days_from_civil(d(1970, 1, 1)), 0);
    assert_eq!(time_fmt::weekday(d(1970, 1, 1)), 4);
    for days in [-1000i64, 0, 1, 20000, 20698, 30000] {
        let date = time_fmt::civil_from_days(days);
        assert_eq!(time_fmt::days_from_civil(date), days);
    }
    assert_eq!(time_fmt::civil_from_days(20698), d(2026, 9, 2));
}
