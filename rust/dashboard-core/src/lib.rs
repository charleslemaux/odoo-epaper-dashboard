//! Pure, no_std, zero-alloc core logic of the e-paper dashboard.
#![cfg_attr(not(test), no_std)]

pub mod bufwrite;
pub mod font_data;
pub mod fonts;
pub mod gfx;
pub mod http;
pub mod icon_data;
pub mod json;
pub mod text;
