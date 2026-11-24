use crate::graphics::font::{get_char_glyph, FONT_HEIGHT, FONT_WIDTH};
use crate::graphics::framebuffer::{Color, GFX};

pub fn fill_rect(x: i32, y: i32, width: u32, height: u32, color: Color) {
    if let Some(ref mut gfx) = *GFX.lock() {
        let screen_w = gfx.width as i32;
        let screen_h = gfx.height as i32;

        let x_start = x.max(0);
        let y_start = y.max(0);
        let x_end = (x + width as i32).min(screen_w);
        let y_end = (y + height as i32).min(screen_h);

        if x_start >= x_end || y_start >= y_end {
            return;
        }

        for py in y_start..y_end {
            for px in x_start..x_end {
                gfx.set_pixel(px as usize, py as usize, color);
            }
        }
    }
}

pub fn draw_rect_outline(x: i32, y: i32, width: u32, height: u32, thickness: u32, color: Color) {
    let t = thickness as i32;
    fill_rect(x, y, width, thickness, color); // Top
    fill_rect(x, y + height as i32 - t, width, thickness, color); // Bottom
    fill_rect(x, y, thickness, height, color); // Left
    fill_rect(x + width as i32 - t, y, thickness, height, color); // Right
}

pub fn draw_gradient_v(x: i32, y: i32, width: u32, height: u32, top: Color, bottom: Color) {
    if height == 0 {
        return;
    }
    for row in 0..height {
        let t = row as u32;
        let r = ((top.r as u32 * (height - t) + bottom.r as u32 * t) / height) as u8;
        let g = ((top.g as u32 * (height - t) + bottom.g as u32 * t) / height) as u8;
        let b = ((top.b as u32 * (height - t) + bottom.b as u32 * t) / height) as u8;
        let color = Color::rgb(r, g, b);
        fill_rect(x, y + row as i32, width, 1, color);
    }
}

pub fn draw_char(x: i32, y: i32, c: char, fg: Color, bg: Option<Color>) {
    if let Some(ref mut gfx) = *GFX.lock() {
        let glyph = get_char_glyph(c);
        let screen_w = gfx.width as i32;
        let screen_h = gfx.height as i32;

        for row in 0..FONT_HEIGHT {
            let py = y + row as i32;
            if py < 0 || py >= screen_h {
                continue;
            }
            let byte = glyph[row];
            for col in 0..FONT_WIDTH {
                let px = x + col as i32;
                if px < 0 || px >= screen_w {
                    continue;
                }
                if (byte & (0x80 >> col)) != 0 {
                    gfx.set_pixel(px as usize, py as usize, fg);
                } else if let Some(bg_color) = bg {
                    gfx.set_pixel(px as usize, py as usize, bg_color);
                }
            }
        }
    }
}

pub fn draw_text(x: i32, y: i32, text: &str, fg: Color, bg: Option<Color>) {
    let mut cur_x = x;
    let mut cur_y = y;

    for c in text.chars() {
        if c == '\n' {
            cur_x = x;
            cur_y += FONT_HEIGHT as i32;
        } else {
            draw_char(cur_x, cur_y, c, fg, bg);
            cur_x += FONT_WIDTH as i32;
        }
    }
}

pub fn draw_text_clipped(x: i32, y: i32, max_width: u32, text: &str, fg: Color, bg: Option<Color>) {
    let mut cur_x = x;
    let max_x = x + max_width as i32;

    for c in text.chars() {
        if cur_x + FONT_WIDTH as i32 > max_x {
            break;
        }
        draw_char(cur_x, y, c, fg, bg);
        cur_x += FONT_WIDTH as i32;
    }
}

pub fn draw_cursor(x: i32, y: i32) {
    // 12x18 modern dark cursor with white outline
    const CURSOR_BITMAP: [&str; 18] = [
        "#...........",
        "##..........",
        "#.#.........",
        "#.##........",
        "#..##.......",
        "#...##......",
        "#....##.....",
        "#.....##....",
        "#......##...",
        "#.......##..",
        "#.....#####.",
        "#..##..##...",
        "#.#..#.##...",
        "##...#.##...",
        "#.....##....",
        "......##....",
        ".......##...",
        "............",
    ];

    if let Some(ref mut gfx) = *GFX.lock() {
        let white = Color::rgb(255, 255, 255);
        let dark = Color::rgb(15, 23, 42);

        for (row, line) in CURSOR_BITMAP.iter().enumerate() {
            let py = y + row as i32;
            if py < 0 || py >= gfx.height as i32 {
                continue;
            }
            for (col, ch) in line.chars().enumerate() {
                let px = x + col as i32;
                if px < 0 || px >= gfx.width as i32 {
                    continue;
                }
                if ch == '#' {
                    gfx.set_pixel(px as usize, py as usize, white);
                } else if ch == '.' && (row > 0 && row < 17 && col > 0 && col < 11) {
                    // Interior dark
                    gfx.set_pixel(px as usize, py as usize, dark);
                }
            }
        }
    }
}
