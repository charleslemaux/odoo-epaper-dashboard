//! Bitmap font containers shared by generated font and icon data.

pub struct Glyph {
    pub offset: u16,
    pub width: u8,
    pub advance: u8,
}

pub struct Font {
    pub height: i32,
    pub glyphs: &'static [Glyph],
    pub bitmap: &'static [u8],
}
