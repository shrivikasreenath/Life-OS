use crate::graphics::canvas::{draw_rect_outline, draw_text, fill_rect};
use crate::graphics::framebuffer::Color;
use alloc::format;
use pc_keyboard::{DecodedKey, KeyCode};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Cell {
    Empty,
    X, // Player
    O, // CPU
}

pub struct TicTacToeGame {
    board: [Cell; 9],
    selected_cell: usize,
    player_turn: bool,
    winner: Option<Cell>,
    is_draw: bool,
    wins: u32,
    losses: u32,
    draws: u32,
}

impl TicTacToeGame {
    pub fn new() -> Self {
        Self {
            board: [Cell::Empty; 9],
            selected_cell: 4, // Center
            player_turn: true,
            winner: None,
            is_draw: false,
            wins: 0,
            losses: 0,
            draws: 0,
        }
    }

    pub fn reset(&mut self) {
        self.board = [Cell::Empty; 9];
        self.selected_cell = 4;
        self.player_turn = true;
        self.winner = None;
        self.is_draw = false;
    }

    pub fn handle_key(&mut self, key: DecodedKey) {
        if self.winner.is_some() || self.is_draw {
            if let DecodedKey::Unicode('r') | DecodedKey::Unicode('R') = key {
                self.reset();
            }
            return;
        }

        match key {
            DecodedKey::RawKey(KeyCode::ArrowUp) => {
                if self.selected_cell >= 3 {
                    self.selected_cell -= 3;
                }
            }
            DecodedKey::RawKey(KeyCode::ArrowDown) => {
                if self.selected_cell + 3 < 9 {
                    self.selected_cell += 3;
                }
            }
            DecodedKey::RawKey(KeyCode::ArrowLeft) => {
                if self.selected_cell % 3 > 0 {
                    self.selected_cell -= 1;
                }
            }
            DecodedKey::RawKey(KeyCode::ArrowRight) => {
                if self.selected_cell % 3 < 2 {
                    self.selected_cell += 1;
                }
            }
            DecodedKey::Unicode('\n') | DecodedKey::Unicode(' ') => {
                self.make_move(self.selected_cell);
            }
            DecodedKey::Unicode(c) if c >= '1' && c <= '9' => {
                let idx = (c as usize) - ('1' as usize);
                self.make_move(idx);
            }
            DecodedKey::Unicode('r') | DecodedKey::Unicode('R') => {
                self.reset();
            }
            _ => {}
        }
    }

    fn make_move(&mut self, idx: usize) {
        if self.board[idx] != Cell::Empty || !self.player_turn {
            return;
        }

        self.board[idx] = Cell::X;
        self.check_game_state();

        if self.winner.is_none() && !self.is_draw {
            self.player_turn = false;
            self.cpu_move();
            self.check_game_state();
            self.player_turn = true;
        }
    }

    fn cpu_move(&mut self) {
        // 1. Can CPU win this move?
        for i in 0..9 {
            if self.board[i] == Cell::Empty {
                self.board[i] = Cell::O;
                if self.evaluate_winner() == Some(Cell::O) {
                    return;
                }
                self.board[i] = Cell::Empty;
            }
        }

        // 2. Can Player win next move? Block them!
        for i in 0..9 {
            if self.board[i] == Cell::Empty {
                self.board[i] = Cell::X;
                if self.evaluate_winner() == Some(Cell::X) {
                    self.board[i] = Cell::O;
                    return;
                }
                self.board[i] = Cell::Empty;
            }
        }

        // 3. Take center if available
        if self.board[4] == Cell::Empty {
            self.board[4] = Cell::O;
            return;
        }

        // 4. Take first available corner or edge
        let order = [0, 2, 6, 8, 1, 3, 5, 7];
        for &idx in &order {
            if self.board[idx] == Cell::Empty {
                self.board[idx] = Cell::O;
                return;
            }
        }
    }

    fn evaluate_winner(&self) -> Option<Cell> {
        const WIN_LINES: [[usize; 3]; 8] = [
            [0, 1, 2], [3, 4, 5], [6, 7, 8], // Rows
            [0, 3, 6], [1, 4, 7], [2, 5, 8], // Cols
            [0, 4, 8], [2, 4, 6],             // Diags
        ];

        for &[a, b, c] in &WIN_LINES {
            if self.board[a] != Cell::Empty
                && self.board[a] == self.board[b]
                && self.board[b] == self.board[c]
            {
                return Some(self.board[a]);
            }
        }
        None
    }

    fn check_game_state(&mut self) {
        if let Some(win) = self.evaluate_winner() {
            self.winner = Some(win);
            if win == Cell::X {
                self.wins += 1;
            } else {
                self.losses += 1;
            }
            return;
        }

        // Check draw
        if self.board.iter().all(|&c| c != Cell::Empty) {
            self.is_draw = true;
            self.draws += 1;
        }
    }

    pub fn render(&self, bx: i32, by: i32, _bw: u32, _bh: u32) {
        let ax = bx + 30;
        let ay = by + 20;
        let cell_size = 90;

        // Draw 3x3 Grid
        fill_rect(ax, ay, (cell_size * 3) as u32, (cell_size * 3) as u32, Color::rgb(9, 13, 22));
        draw_rect_outline(ax, ay, (cell_size * 3) as u32, (cell_size * 3) as u32, 2, Color::rgb(51, 65, 85));

        // Grid dividers
        for i in 1..3 {
            fill_rect(ax + i * cell_size, ay, 2, (cell_size * 3) as u32, Color::rgb(71, 85, 105));
            fill_rect(ax, ay + i * cell_size, (cell_size * 3) as u32, 2, Color::rgb(71, 85, 105));
        }

        // Selected cell highlight
        let sel_row = (self.selected_cell / 3) as i32;
        let sel_col = (self.selected_cell % 3) as i32;
        draw_rect_outline(
            ax + sel_col * cell_size + 4,
            ay + sel_row * cell_size + 4,
            (cell_size - 8) as u32,
            (cell_size - 8) as u32,
            2,
            Color::rgb(6, 182, 212),
        );

        // Cells content
        for row in 0..3 {
            for col in 0..3 {
                let idx = row * 3 + col;
                let cx = ax + (col as i32) * cell_size + 35;
                let cy = ay + (row as i32) * cell_size + 35;

                match self.board[idx] {
                    Cell::X => {
                        draw_text(cx, cy, "X", Color::rgb(6, 182, 212), None);
                    }
                    Cell::O => {
                        draw_text(cx, cy, "O", Color::rgb(239, 68, 68), None);
                    }
                    Cell::Empty => {
                        let hint = format!("{}", idx + 1);
                        draw_text(cx, cy, &hint, Color::rgb(30, 41, 59), None);
                    }
                }
            }
        }

        // Sidebar stats
        let side_x = ax + (cell_size * 3) + 35;
        draw_text(side_x, ay, "TIC-TAC-TOE", Color::rgb(245, 158, 11), None);
        draw_text(side_x, ay + 30, &format!("Player (X) Wins: {}", self.wins), Color::rgb(6, 182, 212), None);
        draw_text(side_x, ay + 55, &format!("CPU (O) Wins   : {}", self.losses), Color::rgb(239, 68, 68), None);
        draw_text(side_x, ay + 80, &format!("Draws          : {}", self.draws), Color::rgb(148, 163, 184), None);

        draw_text(side_x, ay + 130, "Controls:", Color::rgb(148, 163, 184), None);
        draw_text(side_x, ay + 150, "Arrow Keys   - Select Cell", Color::rgb(203, 213, 225), None);
        draw_text(side_x, ay + 170, "Space/Enter  - Place Marker", Color::rgb(203, 213, 225), None);
        draw_text(side_x, ay + 190, "1..9 Keys    - Quick Select", Color::rgb(203, 213, 225), None);
        draw_text(side_x, ay + 210, "R Key        - Restart Game", Color::rgb(203, 213, 225), None);

        if let Some(win) = self.winner {
            fill_rect(ax + 35, ay + 100, 200, 60, Color::rgb(15, 23, 42));
            draw_rect_outline(ax + 35, ay + 100, 200, 60, 2, Color::rgb(16, 185, 129));
            let msg = if win == Cell::X { "YOU WON!" } else { "CPU WON!" };
            draw_text(ax + 95, ay + 112, msg, Color::rgb(16, 185, 129), None);
            draw_text(ax + 55, ay + 134, "Press 'R' to Play Again", Color::rgb(255, 255, 255), None);
        } else if self.is_draw {
            fill_rect(ax + 35, ay + 100, 200, 60, Color::rgb(15, 23, 42));
            draw_rect_outline(ax + 35, ay + 100, 200, 60, 2, Color::rgb(245, 158, 11));
            draw_text(ax + 100, ay + 112, "DRAW!", Color::rgb(245, 158, 11), None);
            draw_text(ax + 55, ay + 134, "Press 'R' to Play Again", Color::rgb(255, 255, 255), None);
        }
    }
}
