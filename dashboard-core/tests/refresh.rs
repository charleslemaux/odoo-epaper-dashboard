use dashboard_core::refresh::{refresh_needed, RefreshTimes, Snapshot};
use dashboard_core::time_fmt::Date;

const DAY1: Date = Date {
    year: 2026,
    month: 10,
    day: 20,
};
const DAY2: Date = Date {
    year: 2026,
    month: 10,
    day: 21,
};

/// Times for a panel that already shows `DAY1`, polled on `DAY1`.
fn same_day(now_s: u32, last_refresh_s: u32) -> RefreshTimes {
    RefreshTimes {
        now_s,
        last_refresh_s,
        has_displayed: true,
        displayed_day: Some(DAY1),
        today: DAY1,
    }
}

#[test]
fn first_display() {
    let a = Snapshot::default();
    let t = RefreshTimes {
        now_s: 30,
        last_refresh_s: 0,
        has_displayed: false,
        displayed_day: None,
        today: DAY1,
    };
    assert!(refresh_needed(&a, &a, &t));
}

#[test]
fn unchanged_skips() {
    let a = Snapshot::default();
    assert!(!refresh_needed(&a, &a, &same_day(10_000, 5_000)));
}

#[test]
fn change_triggers_when_spaced() {
    let a = Snapshot::default();
    let mut b = Snapshot::default();
    b.list.count = 1;
    let _ = b.list.items[0].name.push_str("X");
    assert!(refresh_needed(&a, &b, &same_day(1_000, 500)));
    assert!(!refresh_needed(&a, &b, &same_day(600, 500)));
}

#[test]
fn daily_health_refresh() {
    let a = Snapshot::default();
    assert!(refresh_needed(&a, &a, &same_day(90_000, 100)));
}

#[test]
fn offline_flip_triggers() {
    let a = Snapshot::default();
    let b = Snapshot {
        offline: true,
        ..Default::default()
    };
    assert!(refresh_needed(&a, &b, &same_day(1_000, 500)));
}

#[test]
fn day_change_triggers_with_same_list() {
    let a = Snapshot::default();
    let t = RefreshTimes {
        today: DAY2,
        ..same_day(40_000, 5_000)
    };
    assert!(refresh_needed(&a, &a, &t));
}

#[test]
fn day_change_respects_min_gap() {
    let a = Snapshot::default();
    let t = RefreshTimes {
        today: DAY2,
        ..same_day(5_100, 5_000)
    };
    assert!(!refresh_needed(&a, &a, &t));
}
