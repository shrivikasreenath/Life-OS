use crate::graphics::canvas::{draw_rect_outline, draw_text, draw_text_clipped, fill_rect};
use crate::graphics::framebuffer::Color;
use crate::graphics::icons::{draw_icon, AppIcon};
use alloc::string::String;

pub const TITLEBAR_HEIGHT: u32 = 24;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppId {
    Terminal,
    Browser,
    FileManager,
    NetworkManager,
    Games,
    SysInfo,
    Settings,
    About,
}

pub struct Window {
    pub id: usize,
    pub app_id: AppId,
    pub title: String,
    pub icon: AppIcon,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub is_active: bool,
    pub is_minimized: bool,
    pub is_maximized: bool,
    pub saved_x: i32,
    pub saved_y: i32,
    pub saved_w: u32,
    pub saved_h: u32,
    pub is_dragging: bool,
    pub drag_off_x: i32,
    pub drag_off_y: i32,
}

impl Window {
    pub fn new(
        id: usize,
        app_id: AppId,
        title: &str,
        icon: AppIcon,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            id,
            app_id,
            title: String::from(title),
            icon,
            x,
            y,
            width,
            height,
            is_active: true,
            is_minimized: false,
            is_maximized: false,
            saved_x: x,
            saved_y: y,
            saved_w: width,
            saved_h: height,
            is_dragging: false,
            drag_off_x: 0,
            drag_off_y: 0,
        }
    }

    pub fn body_x(&self) -> i32 {
        self.x + 2
    }

    pub fn body_y(&self) -> i32 {
        self.y + TITLEBAR_HEIGHT as i32
    }

    pub fn body_width(&self) -> u32 {
        if self.width > 4 {
            self.width - 4
        } else {
            0
        }
    }

    pub fn body_height(&self) -> u32 {
        if self.height > TITLEBAR_HEIGHT + 2 {
            self.height - TITLEBAR_HEIGHT - 2
        } else {
            0
        }
    }

    pub fn contains_point(&self, px: i32, py: i32) -> bool {
        if self.is_minimized {
            return false;
        }
        px >= self.x
            && px < (self.x + self.width as i32)
            && py >= self.y
            && py < (self.y + self.height as i32)
    }

    pub fn titlebar_contains(&self, px: i32, py: i32) -> bool {
        if self.is_minimized {
            return false;
        }
        px >= self.x
            && px < (self.x + self.width as i32)
            && py >= self.y
            && py < (self.y + TITLEBAR_HEIGHT as i32)
    }

    pub fn close_button_contains(&self, px: i32, py: i32) -> bool {
        let btn_x = self.x + self.width as i32 - 20;
        let btn_y = self.y + 4;
        px >= btn_x && px <= (btn_x + 14) && py >= btn_y && py <= (btn_y + 14)
    }

    pub fn maximize_button_contains(&self, px: i32, py: i32) -> bool {
        let btn_x = self.x + self.width as i32 - 38;
        let btn_y = self.y + 4;
        px >= btn_x && px <= (btn_x + 14) && py >= btn_y && py <= (btn_y + 14)
    }

    pub fn minimize_button_contains(&self, px: i32, py: i32) -> bool {
        let btn_x = self.x + self.width as i32 - 56;
        let btn_y = self.y + 4;
        px >= btn_x && px <= (btn_x + 14) && py >= btn_y && py <= (btn_y + 14)
    }

    pub fn render_frame(&self) {
        if self.is_minimized {
            return;
        }

        let bg_color = Color::rgb(15, 23, 42); // Slate 900
        let titlebar_color = if self.is_active {
            Color::rgb(30, 41, 59) // Slate 800
        } else {
            Color::rgb(20, 27, 40)
        };
        let border_color = if self.is_active {
            Color::rgb(6, 182, 212) // Cyan 500
        } else {
            Color::rgb(51, 65, 85) // Slate 700
        };

        // Window body background
        fill_rect(self.x, self.y, self.width, self.height, bg_color);

        // Titlebar
        fill_rect(self.x, self.y, self.width, TITLEBAR_HEIGHT, titlebar_color);

        // Active top accent line
        if self.is_active {
            fill_rect(self.x, self.y, self.width, 2, Color::rgb(6, 182, 212));
        }

        // Window icon
        draw_icon(self.x + 4, self.y + 4, self.icon);

        // Window title
        let title_color = if self.is_active {
            Color::rgb(248, 250, 252)
        } else {
            Color::rgb(148, 163, 184)
        };
        let max_title_w = if self.width > 90 { self.width - 90 } else { 0 };
        draw_text_clipped(
            self.x + 24,
            self.y + 5,
            max_title_w,
            &self.title,
            title_color,
            None,
        );

        // Window Controls (Minimize, Maximize, Close)
        // Close (Red)
        fill_rect(
            self.x + self.width as i32 - 20,
            self.y + 5,
            14,
            14,
            Color::rgb(239, 68, 68),
        );
        draw_text(
            self.x + self.width as i32 - 17,
            self.y + 4,
            "x",
            Color::rgb(255, 255, 255),
            None,
        );

        // Maximize (Green)
        fill_rect(
            self.x + self.width as i32 - 38,
            self.y + 5,
            14,
            14,
            Color::rgb(16, 185, 129),
        );
        draw_text(
            self.x + self.width as i32 - 35,
            self.y + 4,
            "+",
            Color::rgb(255, 255, 255),
            None,
        );

        // Minimize (Yellow)
        fill_rect(
            self.x + self.width as i32 - 56,
            self.y + 5,
            14,
            14,
            Color::rgb(245, 158, 11),
        );
        draw_text(
            self.x + self.width as i32 - 53,
            self.y + 4,
            "-",
            Color::rgb(255, 255, 255),
            None,
        );

        // Outer border
        draw_rect_outline(self.x, self.y, self.width, self.height, 1, border_color);
    }
}
