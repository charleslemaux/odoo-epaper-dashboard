//! Refresh-on-change decision policy (port of src/app/refresh.c, plus a
//! redraw on the first poll after local midnight).

use crate::odoo::ActivityList;
use crate::time_fmt::Date;

pub const MIN_GAP_S: u32 = 180;
pub const MAX_AGE_S: u32 = 86_400;

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct Snapshot {
    pub list: ActivityList,
    pub offline: bool,
}

pub struct RefreshTimes {
    pub now_s: u32,
    pub last_refresh_s: u32,
    pub has_displayed: bool,
    /// Local date the panel was last drawn for (`None` before the first
    /// successful draw).
    pub displayed_day: Option<Date>,
    /// Local date at this poll.
    pub today: Date,
}

/// The panel shows date-dependent content (banner date, overdue deadlines,
/// day counters), so a local date change counts as a change just like a
/// different activity list. `MIN_GAP_S` still applies to every redraw but
/// the first one.
#[must_use]
pub fn refresh_needed(prev: &Snapshot, cur: &Snapshot, t: &RefreshTimes) -> bool {
    if !t.has_displayed {
        return true;
    }
    let age = t.now_s.wrapping_sub(t.last_refresh_s);
    let changed = prev != cur || t.displayed_day != Some(t.today);
    (changed || age >= MAX_AGE_S) && age >= MIN_GAP_S
}
