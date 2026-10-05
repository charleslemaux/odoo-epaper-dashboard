//! Pure, no_std, zero-alloc core logic of the e-paper dashboard.
#![cfg_attr(not(test), no_std)]

pub mod activity_icon;
pub mod bufwrite;
pub mod dashboard;
pub mod font_data;
pub mod fonts;
pub mod gfx;
pub mod http;
pub mod icon_data;
pub mod json;
pub mod odoo;
pub mod refresh;
pub mod text;
pub mod time_fmt;
