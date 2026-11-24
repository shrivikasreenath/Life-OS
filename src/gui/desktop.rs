use crate::drivers::e1000;
use crate::drivers::mouse::MouseState;
use crate::drivers::rtc::read_rtc;
use crate::graphics::canvas::{
    draw_gradient_v, draw_rect_outline, draw_text, draw_text_clipped, fill_rect,
};
use crate::graphics::framebuffer::{get_dimensions, Color};
use crate::graphics::icons::{draw_icon, AppIcon};
use crate::gui::window::{AppId, Window};
use alloc::format;
use alloc::vec::Vec;
use spin::Mutex;

pub const TASKBAR_HEIGHT: u32 = 36;
pub const START_MENU_WIDTH: u32 = 250;
pub const START_MENU_HEIGHT: u32 = 320;

pub struct Desktop {
    pub windows: Vec<Window>,
    pub next_window_id: usize,
    pub active_window_id: Option<usize>,
    pub start_menu_open: bool,
    pub prev_mouse: MouseState,
    pub wifi_bridge_connected: bool,
}

pub static DESKTOP: Mutex<Desktop> = Mutex::new(Desktop::new());

impl Desktop {
    pub const fn new() -> Self {
        Self {
            windows: Vec::new(),
            next_window_id: 1,
            active_window_id: None,
            start_menu_open: false,
            prev_mouse: MouseState {
                x: 0,
                y: 0,
                left_button: false,
                right_button: false,
                middle_button: false,
            },
            wifi_bridge_connected: false,
        }
    }

    pub fn open_app(&mut self, app_id: AppId) {
        let (screen_w, screen_h) = get_dimensions();

        // Check if app is already open, if so activate & unminimize it
        let mut found_id = None;
        for w in &mut self.windows {
            if w.app_id == app_id {
                w.is_minimized = false;
                found_id = Some(w.id);
                break;
            }
        }
        if let Some(id) = found_id {
            self.focus_window(id);
            return;
        }

        let (title, icon, w, h) = match app_id {
            AppId::Terminal => ("Terminal — shrivika@life-os", AppIcon::Terminal, 640, 420),
            AppId::Browser => ("Life Web Browser", AppIcon::Browser, 720, 480),
            AppId::FileManager => ("File Manager — LifeFS", AppIcon::FileManager, 600, 400),
            AppId::NetworkManager => ("Network & Wi-Fi Manager", AppIcon::Network, 540, 380),
            AppId::Games => ("Life OS Games Suite (4-in-1)", AppIcon::Games, 600, 440),
            AppId::SysInfo => ("System Information", AppIcon::SysInfo, 520, 400),
            AppId::Settings => ("System Settings", AppIcon::Settings, 500, 380),
            AppId::About => ("About Life OS — Shrivika", AppIcon::About, 480, 360),
        };

        // Center on screen
        let offset = (self.windows.len() as i32 * 25) % 150;
        let x = ((screen_w as i32 - w as i32) / 2 + offset).max(20);
        let y = ((screen_h as i32 - TASKBAR_HEIGHT as i32 - h as i32) / 2 + offset).max(20);

        let id = self.next_window_id;
        self.next_window_id += 1;

        let window = Window::new(id, app_id, title, icon, x, y, w, h);
        self.windows.push(window);
        self.focus_window(id);
        self.start_menu_open = false;
    }

    pub fn close_window(&mut self, id: usize) {
        self.windows.retain(|w| w.id != id);
        if self.active_window_id == Some(id) {
            self.active_window_id = self.windows.last().map(|w| w.id);
            if let Some(new_id) = self.active_window_id {
                self.focus_window(new_id);
            }
        }
    }

    pub fn focus_window(&mut self, id: usize) {
        self.active_window_id = Some(id);
        for w in &mut self.windows {
            w.is_active = w.id == id;
        }
        // Move to back of vector so it renders on top
        if let Some(pos) = self.windows.iter().position(|w| w.id == id) {
            let win = self.windows.remove(pos);
            self.windows.push(win);
        }
    }

    pub fn toggle_minimize(&mut self, id: usize) {
        if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
            w.is_minimized = !w.is_minimized;
            if !w.is_minimized {
                w.is_active = true;
                self.active_window_id = Some(id);
            }
        }
    }

    pub fn toggle_maximize(&mut self, id: usize) {
        let (screen_w, screen_h) = get_dimensions();
        if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
            if w.is_maximized {
                w.x = w.saved_x;
                w.y = w.saved_y;
                w.width = w.saved_w;
                w.height = w.saved_h;
                w.is_maximized = false;
            } else {
                w.saved_x = w.x;
                w.saved_y = w.y;
                w.saved_w = w.width;
                w.saved_h = w.height;
                w.x = 0;
                w.y = 0;
                w.width = screen_w as u32;
                w.height = (screen_h - TASKBAR_HEIGHT as usize) as u32;
                w.is_maximized = true;
            }
        }
    }

    pub fn handle_mouse(&mut self, mouse: MouseState) {
        let (_screen_w, screen_h) = get_dimensions();
        let clicked = mouse.left_button && !self.prev_mouse.left_button;
        let released = !mouse.left_button && self.prev_mouse.left_button;
        let dx = mouse.x - self.prev_mouse.x;
        let dy = mouse.y - self.prev_mouse.y;

        // 1. Handle Window Dragging
        for w in self.windows.iter_mut().rev() {
            if w.is_dragging {
                if released {
                    w.is_dragging = false;
                } else {
                    w.x += dx;
                    w.y += dy;
                }
                self.prev_mouse = mouse;
                return;
            }
        }

        // 2. Click Handling
        if clicked {
            // A. Check Start Button Click
            let tb_y = screen_h as i32 - TASKBAR_HEIGHT as i32;
            if mouse.y >= tb_y && mouse.x >= 4 && mouse.x <= 110 {
                self.start_menu_open = !self.start_menu_open;
                self.prev_mouse = mouse;
                return;
            }

            // B. Check Start Menu Items
            if self.start_menu_open {
                let sm_x = 4;
                let sm_y = tb_y - START_MENU_HEIGHT as i32;
                if mouse.x >= sm_x
                    && mouse.x <= (sm_x + START_MENU_WIDTH as i32)
                    && mouse.y >= sm_y
                    && mouse.y <= tb_y
                {
                    self.handle_start_menu_click(mouse.x - sm_x, mouse.y - sm_y);
                    self.prev_mouse = mouse;
                    return;
                } else {
                    self.start_menu_open = false;
                }
            }

            // C. Check Taskbar Window Tabs
            if mouse.y >= tb_y {
                let mut tab_x = 120;
                let tab_w = 140;
                for w in &self.windows {
                    if mouse.x >= tab_x && mouse.x <= (tab_x + tab_w) {
                        if w.is_minimized || !w.is_active {
                            let wid = w.id;
                            self.toggle_minimize(wid);
                            self.focus_window(wid);
                        } else {
                            let wid = w.id;
                            self.toggle_minimize(wid);
                        }
                        self.prev_mouse = mouse;
                        return;
                    }
                    tab_x += tab_w + 4;
                }
            }

            // D. Check Windows (from top to bottom)
            let mut clicked_win_id = None;
            for w in self.windows.iter_mut().rev() {
                if !w.is_minimized && w.contains_point(mouse.x, mouse.y) {
                    clicked_win_id = Some(w.id);

                    // Check close button
                    if w.close_button_contains(mouse.x, mouse.y) {
                        let wid = w.id;
                        self.close_window(wid);
                        self.prev_mouse = mouse;
                        return;
                    }

                    // Check maximize button
                    if w.maximize_button_contains(mouse.x, mouse.y) {
                        let wid = w.id;
                        self.toggle_maximize(wid);
                        self.prev_mouse = mouse;
                        return;
                    }

                    // Check minimize button
                    if w.minimize_button_contains(mouse.x, mouse.y) {
                        let wid = w.id;
                        self.toggle_minimize(wid);
                        self.prev_mouse = mouse;
                        return;
                    }

                    // Check titlebar drag
                    if w.titlebar_contains(mouse.x, mouse.y) && !w.is_maximized {
                        w.is_dragging = true;
                    } else if mouse.y >= w.body_y() {
                        let local_x = mouse.x - w.body_x();
                        let local_y = mouse.y - w.body_y();
                        crate::apps::dispatch_click(w.app_id, local_x, local_y, w);
                    }

                    break;
                }
            }

            if let Some(id) = clicked_win_id {
                self.focus_window(id);
            }
        }

        self.prev_mouse = mouse;
    }

    fn handle_start_menu_click(&mut self, local_x: i32, local_y: i32) {
        // Items start at y = 60, each item height = 28
        let start_y = 60;
        let item_h = 28;

        if local_y >= start_y && local_y < (start_y + item_h * 8) {
            let index = (local_y - start_y) / item_h;
            match index {
                0 => self.open_app(AppId::Terminal),
                1 => self.open_app(AppId::Browser),
                2 => self.open_app(AppId::FileManager),
                3 => self.open_app(AppId::NetworkManager),
                4 => self.open_app(AppId::Games),
                5 => self.open_app(AppId::SysInfo),
                6 => self.open_app(AppId::Settings),
                7 => self.open_app(AppId::About),
                _ => {}
            }
        } else if local_y >= (START_MENU_HEIGHT as i32 - 32) {
            // Power buttons at bottom
            if local_x >= 10 && local_x <= 110 {
                crate::power::shutdown();
            } else if local_x >= 130 && local_x <= 230 {
                crate::power::reboot();
            }
        }
    }

    pub fn render_wallpaper(&self) {
        let (screen_w, screen_h) = get_dimensions();

        // 1. Vertical Gradient: Deep Navy #0B0F19 to Slate 900 #0F172A
        draw_gradient_v(
            0,
            0,
            screen_w as u32,
            screen_h as u32,
            Color::rgb(11, 15, 25),
            Color::rgb(15, 23, 42),
        );

        // 2. Subtle geometric grid lines
        let grid_color = Color::rgba(30, 41, 59, 100);
        for x in (0..screen_w).step_by(80) {
            fill_rect(x as i32, 0, 1, screen_h as u32, grid_color);
        }
        for y in (0..screen_h).step_by(80) {
            fill_rect(0, y as i32, screen_w as u32, 1, grid_color);
        }

        // 3. Central Brand Typography & Watermark
        let center_x = (screen_w as i32 - 360) / 2;
        let center_y = (screen_h as i32 - TASKBAR_HEIGHT as i32 - 140) / 2;

        // Brand Logo emblem
        draw_icon(center_x + 160, center_y, AppIcon::LifeOsLogo);

        // Title
        draw_text(
            center_x + 120,
            center_y + 24,
            "LIFE OS",
            Color::rgb(6, 182, 212), // Cyan
            None,
        );

        // Subtitle
        draw_text(
            center_x + 20,
            center_y + 48,
            "Autonomous Bare-Metal Rust Operating System",
            Color::rgb(226, 232, 240),
            None,
        );

        // Ownership badge
        draw_text(
            center_x + 60,
            center_y + 72,
            "Owner: Shrivika | Architecture: x86_64",
            Color::rgb(148, 163, 184),
            None,
        );

        // College project badge
        draw_text(
            center_x + 85,
            center_y + 92,
            "College Project Demonstration",
            Color::rgb(16, 185, 129), // Emerald
            None,
        );
    }

    pub fn render_taskbar(&self) {
        let (screen_w, screen_h) = get_dimensions();
        let tb_y = screen_h as i32 - TASKBAR_HEIGHT as i32;

        // Taskbar background
        fill_rect(
            0,
            tb_y,
            screen_w as u32,
            TASKBAR_HEIGHT,
            Color::rgb(15, 23, 42), // Slate 900
        );
        // Top divider
        fill_rect(0, tb_y, screen_w as u32, 1, Color::rgb(51, 65, 85));

        // Start Button
        let start_btn_bg = if self.start_menu_open {
            Color::rgb(6, 182, 212)
        } else {
            Color::rgb(30, 41, 59)
        };
        fill_rect(4, tb_y + 4, 105, TASKBAR_HEIGHT - 8, start_btn_bg);
        draw_icon(8, tb_y + 10, AppIcon::LifeOsLogo);
        draw_text(
            30,
            tb_y + 10,
            "Life OS",
            Color::rgb(255, 255, 255),
            None,
        );

        // Window Tabs
        let mut tab_x = 115;
        let tab_w = 140;
        for w in &self.windows {
            let tab_bg = if w.is_active && !w.is_minimized {
                Color::rgb(30, 41, 59)
            } else {
                Color::rgb(20, 27, 40)
            };
            fill_rect(tab_x, tb_y + 4, tab_w, TASKBAR_HEIGHT - 8, tab_bg);
            if w.is_active && !w.is_minimized {
                fill_rect(tab_x, tb_y + TASKBAR_HEIGHT as i32 - 3, tab_w, 2, Color::rgb(6, 182, 212));
            }
            draw_icon(tab_x + 4, tb_y + 10, w.icon);
            draw_text_clipped(
                tab_x + 24,
                tb_y + 10,
                tab_w - 28,
                &w.title,
                Color::rgb(241, 245, 249),
                None,
            );
            tab_x += tab_w as i32 + 4;
        }

        // System Tray (Right side)
        let tray_x = screen_w as i32 - 240;

        // Ethernet status (e1000)
        let eth_up = e1000::is_up();
        let eth_color = if eth_up { Color::rgb(34, 197, 94) } else { Color::rgb(239, 68, 68) };
        fill_rect(tray_x, tb_y + 14, 8, 8, eth_color);
        draw_text(tray_x + 12, tb_y + 10, "ETH", Color::rgb(203, 213, 225), None);

        // Wi-Fi Host Bridge status
        let _wifi_color = if self.wifi_bridge_connected {
            Color::rgb(34, 197, 94)
        } else {
            Color::rgb(148, 163, 184)
        };
        draw_icon(tray_x + 45, tb_y + 10, AppIcon::Network);

        // Clock & Date
        let dt = read_rtc();
        let time_str = format!("{:02}:{:02}:{:02}", dt.hour, dt.minute, dt.second);
        let date_str = format!("{:04}-{:02}-{:02}", dt.year, dt.month, dt.day);

        draw_text(tray_x + 75, tb_y + 4, &time_str, Color::rgb(255, 255, 255), None);
        draw_text(tray_x + 75, tb_y + 18, &date_str, Color::rgb(148, 163, 184), None);
    }

    pub fn render_start_menu(&self) {
        if !self.start_menu_open {
            return;
        }

        let (_screen_w, screen_h) = get_dimensions();
        let sm_x = 4;
        let sm_y = screen_h as i32 - TASKBAR_HEIGHT as i32 - START_MENU_HEIGHT as i32;

        // Panel background
        fill_rect(
            sm_x,
            sm_y,
            START_MENU_WIDTH,
            START_MENU_HEIGHT,
            Color::rgb(15, 23, 42),
        );
        draw_rect_outline(
            sm_x,
            sm_y,
            START_MENU_WIDTH,
            START_MENU_HEIGHT,
            1,
            Color::rgb(6, 182, 212),
        );

        // Header: User Profile
        fill_rect(sm_x + 1, sm_y + 1, START_MENU_WIDTH - 2, 50, Color::rgb(30, 41, 59));
        // Avatar circle/square
        fill_rect(sm_x + 12, sm_y + 10, 30, 30, Color::rgb(6, 182, 212));
        draw_text(sm_x + 22, sm_y + 17, "S", Color::rgb(255, 255, 255), None);

        draw_text(sm_x + 50, sm_y + 12, "Shrivika", Color::rgb(255, 255, 255), None);
        draw_text(
            sm_x + 50,
            sm_y + 28,
            "shrivika@life-os",
            Color::rgb(148, 163, 184),
            None,
        );

        // Application items
        let items = [
            ("1. Terminal", AppIcon::Terminal),
            ("2. Web Browser", AppIcon::Browser),
            ("3. File Manager", AppIcon::FileManager),
            ("4. Network Manager", AppIcon::Network),
            ("5. Games (4-in-1)", AppIcon::Games),
            ("6. System Info", AppIcon::SysInfo),
            ("7. Settings", AppIcon::Settings),
            ("8. About Life OS", AppIcon::About),
        ];

        let mut item_y = sm_y + 60;
        for (name, icon) in &items {
            draw_icon(sm_x + 12, item_y + 4, *icon);
            draw_text(
                sm_x + 36,
                item_y + 4,
                name,
                Color::rgb(241, 245, 249),
                None,
            );
            item_y += 28;
        }

        // Bottom Power Buttons (Shutdown & Reboot)
        let btn_y = sm_y + START_MENU_HEIGHT as i32 - 32;
        fill_rect(sm_x + 10, btn_y, 100, 24, Color::rgb(239, 68, 68)); // Red Shutdown
        draw_text(sm_x + 26, btn_y + 4, "Shutdown", Color::rgb(255, 255, 255), None);

        fill_rect(sm_x + 130, btn_y, 100, 24, Color::rgb(245, 158, 11)); // Amber Reboot
        draw_text(sm_x + 154, btn_y + 4, "Reboot", Color::rgb(255, 255, 255), None);
    }
}
