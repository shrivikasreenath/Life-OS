use crate::arch::pit;
use crate::graphics::canvas::{draw_rect_outline, draw_text, fill_rect};
use crate::graphics::framebuffer::Color;
use alloc::format;
use alloc::vec::Vec;
use pc_keyboard::{DecodedKey, KeyCode};

const GRID_SIZE: i32 = 20;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub struct SnakeGame {
    snake: Vec<(i32, i32)>,
    dir: Direction,
    next_dir: Direction,
    food: (i32, i32),
    score: u32,
    high_score: u32,
    game_over: bool,
    last_update_tick: u64,
}

impl SnakeGame {
    pub fn new() -> Self {
        let mut game = Self {
            snake: Vec::new(),
            dir: Direction::Right,
            next_dir: Direction::Right,
            food: (15, 10),
            score: 0,
            high_score: 0,
            game_over: false,
            last_update_tick: 0,
        };
        game.reset();
        game
    }

    pub fn reset(&mut self) {
        self.snake.clear();
        self.snake.push((8, 10));
        self.snake.push((7, 10));
        self.snake.push((6, 10));
        self.dir = Direction::Right;
        self.next_dir = Direction::Right;
        self.score = 0;
        self.game_over = false;
        self.spawn_food();
    }

    fn spawn_food(&mut self) {
        let t = pit::get_ticks();
        let fx = ((t % 18) + 1) as i32;
        let fy = (((t / 7) % 18) + 1) as i32;
        self.food = (fx, fy);
    }

    pub fn handle_key(&mut self, key: DecodedKey) {
        match key {
            DecodedKey::RawKey(KeyCode::ArrowUp) => {
                if self.dir != Direction::Down {
                    self.next_dir = Direction::Up;
                }
            }
            DecodedKey::RawKey(KeyCode::ArrowDown) => {
                if self.dir != Direction::Up {
                    self.next_dir = Direction::Down;
                }
            }
            DecodedKey::RawKey(KeyCode::ArrowLeft) => {
                if self.dir != Direction::Right {
                    self.next_dir = Direction::Left;
                }
            }
            DecodedKey::RawKey(KeyCode::ArrowRight) => {
                if self.dir != Direction::Left {
                    self.next_dir = Direction::Right;
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
        if now - self.last_update_tick < 12 || self.game_over {
            return;
        }
        self.last_update_tick = now;

        self.dir = self.next_dir;
        let head = self.snake[0];
        let new_head = match self.dir {
            Direction::Up => (head.0, head.1 - 1),
            Direction::Down => (head.0, head.1 + 1),
            Direction::Left => (head.0 - 1, head.1),
            Direction::Right => (head.0 + 1, head.1),
        };

        // Check wall collision
        if new_head.0 < 0 || new_head.0 >= GRID_SIZE || new_head.1 < 0 || new_head.1 >= GRID_SIZE {
            self.game_over = true;
            return;
        }

        // Check self collision
        if self.snake.contains(&new_head) {
            self.game_over = true;
            return;
        }

        self.snake.insert(0, new_head);

        // Check food collision
        if new_head == self.food {
            self.score += 10;
            if self.score > self.high_score {
                self.high_score = self.score;
            }
            self.spawn_food();
        } else {
            self.snake.pop();
        }
    }

    pub fn render(&self, bx: i32, by: i32, _bw: u32, _bh: u32) {
        // Arena: 360x360
        let cell_size = 18;
        let arena_w = (GRID_SIZE * cell_size) as u32;
        let arena_h = (GRID_SIZE * cell_size) as u32;
        let ax = bx + 20;
        let ay = by + 10;

        // Draw Arena Board
        fill_rect(ax, ay, arena_w, arena_h, Color::rgb(9, 13, 22));
        draw_rect_outline(ax, ay, arena_w, arena_h, 2, Color::rgb(51, 65, 85));

        // Draw Food
        fill_rect(
            ax + self.food.0 * cell_size + 2,
            ay + self.food.1 * cell_size + 2,
            (cell_size - 4) as u32,
            (cell_size - 4) as u32,
            Color::rgb(239, 68, 68), // Red
        );

        // Draw Snake
        for (i, seg) in self.snake.iter().enumerate() {
            let color = if i == 0 {
                Color::rgb(6, 182, 212) // Cyan head
            } else {
                Color::rgb(16, 185, 129) // Emerald body
            };
            fill_rect(
                ax + seg.0 * cell_size + 1,
                ay + seg.1 * cell_size + 1,
                (cell_size - 2) as u32,
                (cell_size - 2) as u32,
                color,
            );
        }

        // Sidebar stats
        let side_x = ax + arena_w as i32 + 25;
        draw_text(side_x, ay + 10, "SNAKE CLASSIC", Color::rgb(245, 158, 11), None);
        draw_text(side_x, ay + 40, &format!("Score: {}", self.score), Color::rgb(255, 255, 255), None);
        draw_text(side_x, ay + 65, &format!("High Score: {}", self.high_score), Color::rgb(16, 185, 129), None);

        draw_text(side_x, ay + 120, "Controls:", Color::rgb(148, 163, 184), None);
        draw_text(side_x, ay + 140, "Arrow Keys - Move", Color::rgb(203, 213, 225), None);
        draw_text(side_x, ay + 160, "R Key      - Restart", Color::rgb(203, 213, 225), None);

        if self.game_over {
            fill_rect(ax + 80, ay + 140, 200, 60, Color::rgb(15, 23, 42));
            draw_rect_outline(ax + 80, ay + 140, 200, 60, 2, Color::rgb(239, 68, 68));
            draw_text(ax + 125, ay + 150, "GAME OVER!", Color::rgb(239, 68, 68), None);
            draw_text(ax + 105, ay + 172, "Press 'R' to Restart", Color::rgb(255, 255, 255), None);
        }
    }
}
