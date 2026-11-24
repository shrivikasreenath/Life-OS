use crate::arch::pit;
use crate::graphics::canvas::{draw_rect_outline, draw_text, fill_rect};
use crate::graphics::framebuffer::Color;
use alloc::format;
use pc_keyboard::{DecodedKey, KeyCode};

pub struct PongGame {
    player_y: i32,
    cpu_y: i32,
    ball_x: i32,
    ball_y: i32,
    ball_vx: i32,
    ball_vy: i32,
    player_score: u32,
    cpu_score: u32,
    game_over: bool,
    winner: Option<&'static str>,
    last_tick: u64,
}

impl PongGame {
    pub fn new() -> Self {
        Self {
            player_y: 120,
            cpu_y: 120,
            ball_x: 200,
            ball_y: 150,
            ball_vx: 4,
            ball_vy: 3,
            player_score: 0,
            cpu_score: 0,
            game_over: false,
            winner: None,
            last_tick: 0,
        }
    }

    pub fn reset(&mut self) {
        self.player_score = 0;
        self.cpu_score = 0;
        self.game_over = false;
        self.winner = None;
        self.reset_ball(4);
    }

    fn reset_ball(&mut self, vx: i32) {
        self.ball_x = 200;
        self.ball_y = 150;
        self.ball_vx = vx;
        self.ball_vy = 3;
    }

    pub fn handle_key(&mut self, key: DecodedKey) {
        match key {
            DecodedKey::RawKey(KeyCode::ArrowUp) | DecodedKey::Unicode('w') | DecodedKey::Unicode('W') => {
                if self.player_y > 10 {
                    self.player_y -= 16;
                }
            }
            DecodedKey::RawKey(KeyCode::ArrowDown) | DecodedKey::Unicode('s') | DecodedKey::Unicode('S') => {
                if self.player_y < 280 {
                    self.player_y += 16;
                }
            }
            DecodedKey::Unicode('r') | DecodedKey::Unicode('R') => {
                self.reset();
            }
            _ => {}
        }
    }

    pub fn update(&mut self) {
        let now = pit::get_ticks();
        if now - self.last_tick < 3 || self.game_over {
            return;
        }
        self.last_tick = now;

        // Ball movement
        self.ball_x += self.ball_vx;
        self.ball_y += self.ball_vy;

        // Top/Bottom wall bounce
        if self.ball_y <= 10 {
            self.ball_y = 10;
            self.ball_vy = -self.ball_vy;
        } else if self.ball_y >= 310 {
            self.ball_y = 310;
            self.ball_vy = -self.ball_vy;
        }

        // CPU AI tracking
        let cpu_center = self.cpu_y + 30;
        if cpu_center < self.ball_y - 8 && self.cpu_y < 280 {
            self.cpu_y += 3;
        } else if cpu_center > self.ball_y + 8 && self.cpu_y > 10 {
            self.cpu_y -= 3;
        }

        // Player paddle collision (x = 30, w = 10, h = 60)
        if self.ball_x <= 40 && self.ball_x >= 25 {
            if self.ball_y >= self.player_y - 6 && self.ball_y <= self.player_y + 66 {
                self.ball_vx = self.ball_vx.abs(); // Bounce right
                self.ball_x = 41;
            }
        }

        // CPU paddle collision (x = 360, w = 10, h = 60)
        if self.ball_x >= 350 && self.ball_x <= 365 {
            if self.ball_y >= self.cpu_y - 6 && self.ball_y <= self.cpu_y + 66 {
                self.ball_vx = -self.ball_vx.abs(); // Bounce left
                self.ball_x = 349;
            }
        }

        // Scoring
        if self.ball_x < 10 {
            self.cpu_score += 1;
            if self.cpu_score >= 7 {
                self.game_over = true;
                self.winner = Some("CPU Wins!");
            } else {
                self.reset_ball(4);
            }
        } else if self.ball_x > 390 {
            self.player_score += 1;
            if self.player_score >= 7 {
                self.game_over = true;
                self.winner = Some("Player Wins!");
            } else {
                self.reset_ball(-4);
            }
        }
    }

    pub fn render(&self, bx: i32, by: i32, _bw: u32, _bh: u32) {
        let ax = bx + 20;
        let ay = by + 10;
        let arena_w = 400;
        let arena_h = 320;

        // Arena Board
        fill_rect(ax, ay, arena_w, arena_h, Color::rgb(9, 13, 22));
        draw_rect_outline(ax, ay, arena_w, arena_h, 2, Color::rgb(51, 65, 85));

        // Center dashed line
        for dy in (ay + 8..ay + arena_h as i32 - 8).step_by(16) {
            fill_rect(ax + 200, dy, 2, 8, Color::rgb(51, 65, 85));
        }

        // Player Paddle (Cyan)
        fill_rect(ax + 30, ay + self.player_y, 10, 60, Color::rgb(6, 182, 212));

        // CPU Paddle (Red)
        fill_rect(ax + 360, ay + self.cpu_y, 10, 60, Color::rgb(239, 68, 68));

        // Ball (White)
        fill_rect(ax + self.ball_x, ay + self.ball_y, 8, 8, Color::rgb(255, 255, 255));

        // Score display
        draw_text(ax + 160, ay + 15, &format!("{}", self.player_score), Color::rgb(6, 182, 212), None);
        draw_text(ax + 230, ay + 15, &format!("{}", self.cpu_score), Color::rgb(239, 68, 68), None);

        // Sidebar
        let side_x = ax + arena_w as i32 + 25;
        draw_text(side_x, ay + 10, "RETRO PONG", Color::rgb(245, 158, 11), None);
        draw_text(side_x, ay + 40, "First to 7 wins!", Color::rgb(203, 213, 225), None);
        draw_text(side_x, ay + 80, "Controls:", Color::rgb(148, 163, 184), None);
        draw_text(side_x, ay + 100, "W / Up   - Move Up", Color::rgb(203, 213, 225), None);
        draw_text(side_x, ay + 120, "S / Down - Move Down", Color::rgb(203, 213, 225), None);
        draw_text(side_x, ay + 140, "R Key    - Restart", Color::rgb(203, 213, 225), None);

        if self.game_over {
            fill_rect(ax + 100, ay + 120, 200, 70, Color::rgb(15, 23, 42));
            draw_rect_outline(ax + 100, ay + 120, 200, 70, 2, Color::rgb(245, 158, 11));
            let win_msg = self.winner.unwrap_or("Match Over");
            draw_text(ax + 150, ay + 135, win_msg, Color::rgb(245, 158, 11), None);
            draw_text(ax + 125, ay + 160, "Press 'R' to Restart", Color::rgb(255, 255, 255), None);
        }
    }
}
