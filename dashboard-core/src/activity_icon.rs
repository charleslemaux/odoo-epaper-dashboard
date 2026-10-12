//! Mapping from Odoo activity icon classes to embedded icons
//! (port of `src/app/activity_icon.c`).

use crate::icon_data::Icon;

const RULES: &[(&str, Icon)] = &[
    ("fa-check", Icon::Check),
    ("fa-envelope", Icon::Envelope),
    ("fa-phone", Icon::Phone),
    ("fa-users", Icon::Users),
    ("fa-inbox", Icon::Inbox),
    ("fa-code", Icon::Code),
    ("fa-cube", Icon::Cube),
    ("fa-microchip", Icon::Microchip),
    ("fa-lightbulb-o", Icon::Lightbulb),
    ("fa-calendar-check-o", Icon::CalendarCheck),
    ("fa-calendar", Icon::CalendarCheck),
    ("fa-upload", Icon::Upload),
    ("fa-pencil-square-o", Icon::PenSquare),
    ("fa-pencil", Icon::PenSquare),
    ("fa-tasks", Icon::Check),
    ("fa-clock-o", Icon::Clock),
];

#[must_use]
pub fn icon_for(class: &str) -> Icon {
    RULES
        .iter()
        .find(|(needle, _)| *needle == class)
        .map_or(Icon::Clock, |(_, ic)| *ic)
}
