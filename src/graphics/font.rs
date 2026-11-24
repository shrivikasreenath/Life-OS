pub const FONT_WIDTH: usize = 8;
pub const FONT_HEIGHT: usize = 16;

include!("font_data.rs");

pub fn get_char_glyph(c: char) -> &'static [u8; 16] {
    let ascii = c as usize;
    if ascii >= 32 && ascii < 127 {
        &BASIC_FONT[ascii - 32]
    } else {
        &UNKNOWN_GLYPH
    }
}
