use crate::arch::pit;
use crate::graphics::canvas::{draw_rect_outline, draw_text, fill_rect};
use crate::graphics::framebuffer::Color;
use alloc::format;
use pc_keyboard::{DecodedKey, KeyCode};

pub struct Game2048 {
    grid: [[u32; 4]; 4],
    score: u32,
    high_score: u32,
    game_over: bool,
    won: bool,
}

impl Game2048 {
    pub fn new() -> Self {
        let mut game = Self {
            grid: [[0; 4]; 4],
            score: 0,
            high_score: 0,
            game_over: false,
            won: false,
        };
        game.reset();
        game
    }

    pub fn reset(&mut self) {
        self.grid = [[0; 4]; 4];
        self.score = 0;
        self.game_over = false;
        self.won = false;
        self.spawn_tile();
        self.spawn_tile();
    }

    fn spawn_tile(&mut self) {
        let mut empty = [(0, 0); 16];
        let mut count = 0;

        for r in 0..4 {
            for c in 0..4 {
                if self.grid[r][c] == 0 {
                    empty[count] = (r, c);
                    count += 1;
                }
            }
        }

        if count == 0 {
            return;
        }

        let seed = pit::get_ticks();
        let idx = (seed as usize) % count;
        let (r, c) = empty[idx];
        let val = if (seed % 10) == 0 { 4 } else { 2 };
        self.grid[r][c] = val;
    }

    pub fn handle_key(&mut self, key: DecodedKey) {
        if self.game_over {
            if let DecodedKey::Unicode('r') | DecodedKey::Unicode('R') = key {
                self.reset();
            }
            return;
        }

        let moved = match key {
            DecodedKey::RawKey(KeyCode::ArrowLeft) => self.slide_left(),
            DecodedKey::RawKey(KeyCode::ArrowRight) => self.slide_right(),
            DecodedKey::RawKey(KeyCode::ArrowUp) => self.slide_up(),
            DecodedKey::RawKey(KeyCode::ArrowDown) => self.slide_down(),
            DecodedKey::Unicode('r') | DecodedKey::Unicode('R') => {
                self.reset();
                false
            }
            _ => false,
        };

        if moved {
            self.spawn_tile();
            if self.score > self.high_score {
                self.high_score = self.score;
            }
            self.check_game_over();
        }
    }

    fn slide_row_left(&mut self, row: &mut [u32; 4]) -> bool {
        let mut changed = false;
        let original = *row;

        // 1. Shift non-zeros left
        let mut compact = [0u32; 4];
        let mut pos = 0;
        for &val in row.iter() {
            if val != 0 {
                compact[pos] = val;
                pos += 1;
            }
        }

        // 2. Merge adjacent equals
        let mut merged = [0u32; 4];
        let mut m_pos = 0;
        let mut i = 0;
        while i < 4 {
            if i + 1 < 4 && compact[i] != 0 && compact[i] == compact[i + 1] {
                let new_val = compact[i] * 2;
                merged[m_pos] = new_val;
                self.score += new_val;
                if new_val == 2048 {
                    self.won = true;
                }
                m_pos += 1;
                i += 2;
            } else if compact[i] != 0 {
                merged[m_pos] = compact[i];
                m_pos += 1;
                i += 1;
            } else {
                i += 1;
            }
        }

        *row = merged;
        changed = *row != original;
        changed
    }

    fn slide_left(&mut self) -> bool {
        let mut moved = false;
        for r in 0..4 {
            let mut row = self.grid[r];
            if self.slide_row_left(&mut row) {
                self.grid[r] = row;
                moved = true;
            }
        }
        moved
    }

    fn slide_right(&mut self) -> bool {
        let mut moved = false;
        for r in 0..4 {
            let mut row = [self.grid[r][3], self.grid[r][2], self.grid[r][1], self.grid[r][0]];
            if self.slide_row_left(&mut row) {
                self.grid[r] = [row[3], row[2], row[1], row[0]];
                moved = true;
            }
        }
        moved
    }

    fn slide_up(&mut self) -> bool {
        let mut moved = false;
        for c in 0..4 {
            let mut col = [self.grid[0][c], self.grid[1][c], self.grid[2][c], self.grid[3][c]];
            if self.slide_row_left(&mut col) {
                for r in 0..4 {
                    self.grid[r][c] = col[r];
                }
                moved = true;
            }
        }
        moved
    }

    fn slide_down(&mut self) -> bool {
        let mut moved = false;
        for c in 0..4 {
            let mut col = [self.grid[3][c], self.grid[2][c], self.grid[1][c], self.grid[0][c]];
            if self.slide_row_left(&mut col) {
                for r in 0..4 {
                    self.grid[3 - r][c] = col[r];
                }
                moved = true;
            }
        }
        moved
    }

    fn check_game_over(&mut self) {
        // Any empty cell?
        for r in 0..4 {
            for c in 0..4 {
                if self.grid[r][c] == 0 {
                    return;
                }
            }
        }

        // Any adjacent merges possible?
        for r in 0..4 {
            for c in 0..4 {
                let v = self.grid[r][c];
                if c + 1 < 4 && self.grid[r][c + 1] == v {
                    return;
                }
                if r + 1 < 4 && self.grid[r + 1][c] == v {
                    return;
                }
            }
        }

        self.game_over = true;
    }

    fn tile_color(&self, val: u32) -> Color {
        match val {
            2 => Color::rgb(238, 228, 218),
            4 => Color::rgb(237, 224, 200),
            8 => Color::rgb(242, 177, 121),
            16 => Color::rgb(245, 149, 99),
            32 => Color::rgb(246, 124, 95),
            64 => Color::rgb(246, 94, 59),
            128 => Color::rgb(237, 207, 114),
            256 => Color::rgb(237, 204, 97),
            512 => Color::rgb(237, 200, 80),
            1024 => Color::rgb(237, 197, 63),
            2048 => Color::rgb(237, 194, 46),
            _ => Color::rgb(30, 41, 59),
        }
    }

    pub fn render(&self, bx: i32, by: i32, _bw: u32, _bh: u32) {
        let ax = bx + 30;
        let ay = by + 20;
        let tile_size = 64;
        let pad = 8;
        let board_size = (tile_size * 4 + pad * 5) as u32;

        // Board background
        fill_rect(ax, ay, board_size, board_size, Color::rgb(187, 173, 160));

        // Draw Tiles
        for r in 0..4 {
            for c in 0..4 {
                let tx = ax + pad + (c as i32) * (tile_size + pad);
                let ty = ay + pad + (r as i32) * (tile_size + pad);
                let val = self.grid[r][c];

                let bg = if val == 0 {
                    Color::rgb(205, 193, 180)
                } else {
                    self.tile_color(val)
                };

                fill_rect(tx, ty, tile_size as u32, tile_size as u32, bg);

                if val > 0 {
                    let text = format!("{}", val);
                    let fg = if val <= 4 {
                        Color::rgb(119, 110, 101)
                    } else {
                        Color::rgb(255, 255, 255)
                    };
                    let text_x = tx + (tile_size - (text.len() * 8) as i32) / 2;
                    let text_y = ty + (tile_size - 16) / 2;
                    draw_text(text_x, text_y, &text, fg, None);
                }
            }
        }

        // Sidebar
        let side_x = ax + board_size as i32 + 25;
        draw_text(side_x, ay, "2048 PUZZLE", Color::rgb(245, 158, 11), None);
        draw_text(side_x, ay + 30, &format!("Current Score: {}", self.score), Color::rgb(255, 255, 255), None);
        draw_text(side_x, ay + 55, &format!("Best Score   : {}", self.high_score), Color::rgb(16, 185, 129), None);

        draw_text(side_x, ay + 110, "Controls:", Color::rgb(148, 163, 184), None);
        draw_text(side_x, ay + 130, "Arrow Keys - Slide Tiles", Color::rgb(203, 213, 225), None);
        draw_text(side_x, ay + 150, "R Key      - Restart Game", Color::rgb(203, 213, 225), None);

        draw_text(side_x, ay + 190, "Goal: Join the numbers", Color::rgb(148, 163, 184), None);
        draw_text(side_x, ay + 210, "and get to the 2048 tile!", Color::rgb(148, 163, 184), None);

        if self.game_over {
            fill_rect(ax + 50, ay + 120, 200, 60, Color::rgb(15, 23, 42));
            draw_rect_outline(ax + 50, ay + 120, 200, 60, 2, Color::rgb(239, 68, 68));
            draw_text(ax + 95, ay + 132, "GAME OVER!", Color::rgb(239, 68, 68), None);
            draw_text(ax + 75, ay + 154, "Press 'R' to Restart", Color::rgb(255, 255, 255), None);
        } else if self.won {
            fill_rect(ax + 50, ay + 120, 200, 60, Color::rgb(15, 23, 42));
            draw_rect_outline(ax + 50, ay + 120, 200, 60, 2, Color::rgb(245, 158, 11));
            draw_text(ax + 85, ay + 132, "YOU REACHED 2048!", Color::rgb(245, 158, 11), None);
            draw_text(ax + 75, ay + 154, "Keep playing or press 'R'", Color::rgb(255, 255, 255), None);
        }
    }
}
