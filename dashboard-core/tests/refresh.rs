use dashboard_core::refresh::{refresh_needed, RefreshTimes, Snapshot};

#[test]
fn first_display() {
    let a = Snapshot::default();
    let t = RefreshTimes {
        now_s: 30,
        last_refresh_s: 0,
        has_displayed: false,
    };
    assert!(refresh_needed(&a, &a, &t));
}

#[test]
fn unchanged_skips() {
    let a = Snapshot::default();
    let t = RefreshTimes {
        now_s: 10_000,
        last_refresh_s: 5_000,
        has_displayed: true,
    };
    assert!(!refresh_needed(&a, &a, &t));
}

#[test]
fn change_triggers_when_spaced() {
    let a = Snapshot::default();
    let mut b = Snapshot::default();
    b.list.count = 1;
    let _ = b.list.items[0].name.push_str("X");
    let spaced = RefreshTimes {
        now_s: 1_000,
        last_refresh_s: 500,
        has_displayed: true,
    };
    let close = RefreshTimes {
        now_s: 600,
        last_refresh_s: 500,
        has_displayed: true,
    };
    assert!(refresh_needed(&a, &b, &spaced));
    assert!(!refresh_needed(&a, &b, &close));
}

#[test]
fn daily_health_refresh() {
    let a = Snapshot::default();
    let t = RefreshTimes {
        now_s: 90_000,
        last_refresh_s: 100,
        has_displayed: true,
    };
    assert!(refresh_needed(&a, &a, &t));
}

#[test]
fn offline_flip_triggers() {
    let a = Snapshot::default();
    let b = Snapshot {
        offline: true,
        ..Default::default()
    };
    let t = RefreshTimes {
        now_s: 1_000,
        last_refresh_s: 500,
        has_displayed: true,
    };
    assert!(refresh_needed(&a, &b, &t));
}
