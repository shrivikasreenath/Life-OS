pub mod game_2048;
pub mod pong;
pub mod snake;
pub mod tictactoe;

use crate::graphics::canvas::{draw_text, fill_rect};
use crate::graphics::framebuffer::Color;
use crate::gui::window::Window;
use game_2048::Game2048;
use pc_keyboard::DecodedKey;
use pong::PongGame;
use snake::SnakeGame;
use spin::Mutex;
use tictactoe::TicTacToeGame;

pub struct GamesSuite {
    pub current_game: usize, // 0 = Snake, 1 = Pong, 2 = Tic-Tac-Toe, 3 = 2048
    pub snake: SnakeGame,
    pub pong: PongGame,
    pub tictactoe: TicTacToeGame,
    pub game_2048: Game2048,
}

pub static GAMES_SUITE: Mutex<Option<GamesSuite>> = Mutex::new(None);

pub fn init() {
    *GAMES_SUITE.lock() = Some(GamesSuite::new());
}

impl GamesSuite {
    pub fn new() -> Self {
        Self {
            current_game: 0,
            snake: SnakeGame::new(),
            pong: PongGame::new(),
            tictactoe: TicTacToeGame::new(),
            game_2048: Game2048::new(),
        }
    }

    pub fn handle_key(&mut self, key: DecodedKey) {
        match key {
            DecodedKey::Unicode('1') => self.current_game = 0,
            DecodedKey::Unicode('2') => self.current_game = 1,
            DecodedKey::Unicode('3') => self.current_game = 2,
            DecodedKey::Unicode('4') => self.current_game = 3,
            _ => {
                match self.current_game {
                    0 => self.snake.handle_key(key),
                    1 => self.pong.handle_key(key),
                    2 => self.tictactoe.handle_key(key),
                    3 => self.game_2048.handle_key(key),
                    _ => {}
                }
            }
        }
    }

    pub fn update(&mut self) {
        match self.current_game {
            0 => self.snake.update(),
            1 => self.pong.update(),
            _ => {}
        }
    }

    pub fn handle_click(&mut self, local_x: i32, local_y: i32) {
        if local_y < 30 {
            if local_x >= 10 && local_x <= 110 {
                self.current_game = 0; // Snake
            } else if local_x >= 120 && local_x <= 210 {
                self.current_game = 1; // Pong
            } else if local_x >= 220 && local_x <= 340 {
                self.current_game = 2; // Tic-Tac-Toe
            } else if local_x >= 350 && local_x <= 440 {
                self.current_game = 3; // 2048
            }
        }
    }

    pub fn render(&self, win: &Window) {
        let bx = win.body_x();
        let by = win.body_y();
        let bw = win.body_width();
        let bh = win.body_height();

        // Game Suite background
        fill_rect(bx, by, bw, bh, Color::rgb(15, 23, 42));

        // Top Game Tabs
        fill_rect(bx, by, bw, 30, Color::rgb(30, 41, 59));

        let tabs = [
            ("1. Snake", 10, 100),
            ("2. Pong", 120, 90),
            ("3. Tic-Tac-Toe", 220, 120),
            ("4. 2048", 350, 90),
        ];

        for (i, (name, x, w)) in tabs.iter().enumerate() {
            let bg = if self.current_game == i {
                Color::rgb(6, 182, 212)
            } else {
                Color::rgb(51, 65, 85)
            };
            fill_rect(bx + x, by + 3, *w, 24, bg);
            draw_text(bx + x + 8, by + 7, name, Color::rgb(255, 255, 255), None);
        }

        let content_y = by + 32;
        let content_h = if bh > 32 { bh - 32 } else { 0 };

        match self.current_game {
            0 => self.snake.render(bx, content_y, bw, content_h),
            1 => self.pong.render(bx, content_y, bw, content_h),
            2 => self.tictactoe.render(bx, content_y, bw, content_h),
            3 => self.game_2048.render(bx, content_y, bw, content_h),
            _ => {}
        }
    }
}
