//! Refresh-on-change decision policy (port of src/app/refresh.c).

use crate::odoo::ActivityList;

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
}

#[must_use]
pub fn refresh_needed(prev: &Snapshot, cur: &Snapshot, t: &RefreshTimes) -> bool {
    if !t.has_displayed {
        return true;
    }
    let age = t.now_s.wrapping_sub(t.last_refresh_s);
    let changed = prev != cur;
    (changed || age >= MAX_AGE_S) && age >= MIN_GAP_S
}
