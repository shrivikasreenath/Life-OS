pub const FONT_WIDTH: usize = 8;
pub const FONT_HEIGHT: usize = 16;

include!("font_data.rs");

pub fn get_char_glyph(c: char) -> &'static [u8; 16] {
    let mapped = match c {
        '\u{2018}' | '\u{2019}' | '`' | '\u{00B4}' => '\'',
        '\u{201C}' | '\u{201D}' | '\u{00AB}' | '\u{00BB}' => '"',
        '\u{2013}' | '\u{2014}' | '\u{2212}' => '-',
        '\u{2026}' => '.',
        '\u{00A0}' => ' ',
        _ => c,
    };
    let ascii = mapped as usize;
    if ascii >= 32 && ascii < 127 {
        &BASIC_FONT[ascii - 32]
    } else {
        &UNKNOWN_GLYPH
    }
}
