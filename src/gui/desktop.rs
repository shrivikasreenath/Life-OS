use crate::arch::pit;
use crate::drivers::e1000;
use crate::drivers::mouse::MouseState;
use crate::drivers::rtc::read_rtc;
use crate::graphics::canvas::{
    draw_gradient_v, draw_rect_outline, draw_text, draw_text_clipped, fill_rect,
};
use crate::graphics::framebuffer::{get_dimensions, Color};
use crate::graphics::icons::{draw_icon, draw_icon_large, AppIcon};
use crate::gui::window::{AppId, Window};
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;

pub const TASKBAR_HEIGHT: u32 = 38;
pub const START_MENU_WIDTH: u32 = 270;
pub const START_MENU_HEIGHT: u32 = 360;
pub const ICON_SIZE: i32 = 32;
pub const ICON_CELL_W: i32 = 76;
pub const ICON_CELL_H: i32 = 72;

#[derive(Clone)]
pub struct DesktopIconItem {
    pub app_id: Option<AppId>,
    pub name: &'static str,
    pub icon: AppIcon,
    pub x: i32,
    pub y: i32,
    pub is_selected: bool,
}

pub struct Desktop {
    pub windows: Vec<Window>,
    pub next_window_id: usize,
    pub active_window_id: Option<usize>,
    pub start_menu_open: bool,
    pub start_menu_search: String,
    pub prev_mouse: MouseState,
    pub wifi_bridge_connected: bool,
    pub current_wifi_ssid: String,
    pub wifi_signal: u8,
    pub volume_muted: bool,
    pub calendar_open: bool,
    pub context_menu_open: bool,
    pub context_menu_x: i32,
    pub context_menu_y: i32,
    pub wallpaper_theme: usize, // 0 = Cyber Grid, 1 = Deep Cobalt Aurora, 2 = Midnight Slate
    pub icons: Vec<DesktopIconItem>,
    pub dragged_icon_idx: Option<usize>,
    pub drag_offset_x: i32,
    pub drag_offset_y: i32,
    pub last_click_time_ms: u64,
    pub last_click_icon_idx: Option<usize>,
}

pub static DESKTOP: Mutex<Desktop> = Mutex::new(Desktop::new());

impl Desktop {
    pub const fn new() -> Self {
        Self {
            windows: Vec::new(),
            next_window_id: 1,
            active_window_id: None,
            start_menu_open: false,
            start_menu_search: String::new(),
            prev_mouse: MouseState {
                x: 0,
                y: 0,
                left_button: false,
                right_button: false,
                middle_button: false,
            },
            wifi_bridge_connected: false,
            current_wifi_ssid: String::new(),
            wifi_signal: 85,
            volume_muted: false,
            calendar_open: false,
            context_menu_open: false,
            context_menu_x: 0,
            context_menu_y: 0,
            wallpaper_theme: 0,
            icons: Vec::new(),
            dragged_icon_idx: None,
            drag_offset_x: 0,
            drag_offset_y: 0,
            last_click_time_ms: 0,
            last_click_icon_idx: None,
        }
    }

    pub fn init_icons(&mut self) {
        if !self.icons.is_empty() {
            return;
        }

        // Initialize 8 required desktop application icons in left grid
        let items: [(&'static str, Option<AppId>, AppIcon); 8] = [
            ("Web Browser", Some(AppId::Browser), AppIcon::Browser),
            ("Terminal", Some(AppId::Terminal), AppIcon::Terminal),
            ("File Manager", Some(AppId::FileManager), AppIcon::FileManager),
            ("Network", Some(AppId::NetworkManager), AppIcon::Network),
            ("Games", Some(AppId::Games), AppIcon::Games),
            ("System Info", Some(AppId::SysInfo), AppIcon::SysInfo),
            ("Settings", Some(AppId::Settings), AppIcon::Settings),
            ("Trash", None, AppIcon::Trash),
        ];

        let mut y = 20;
        let mut x = 20;
        for (name, app_id, icon) in items {
            self.icons.push(DesktopIconItem {
                app_id,
                name,
                icon,
                x,
                y,
                is_selected: false,
            });
            y += ICON_CELL_H + 12;
            if y > 560 {
                y = 20;
                x += ICON_CELL_W + 16;
            }
        }
    }

    pub fn arrange_icons(&mut self) {
        let mut y = 20;
        let mut x = 20;
        for icon in &mut self.icons {
            icon.x = x;
            icon.y = y;
            icon.is_selected = false;
            y += ICON_CELL_H + 12;
            if y > 560 {
                y = 20;
                x += ICON_CELL_W + 16;
            }
        }
    }

    pub fn cycle_wallpaper(&mut self) {
        self.wallpaper_theme = (self.wallpaper_theme + 1) % 3;
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
            self.start_menu_open = false;
            self.context_menu_open = false;
            return;
        }

        let (title, icon, w, h) = match app_id {
            AppId::Terminal => ("Terminal — shrivika@life-os", AppIcon::Terminal, 640, 420),
            AppId::Browser => ("Life Web Browser", AppIcon::Browser, 720, 480),
            AppId::FileManager => ("File Manager — LifeFS", AppIcon::FileManager, 620, 420),
            AppId::NetworkManager => ("Network & Wi-Fi Manager", AppIcon::Network, 580, 400),
            AppId::Games => ("Life OS Games Suite (4-in-1)", AppIcon::Games, 600, 440),
            AppId::SysInfo => ("System Information", AppIcon::SysInfo, 540, 400),
            AppId::Settings => ("System Settings", AppIcon::Settings, 500, 380),
            AppId::About => ("About Life OS — Shrivika", AppIcon::About, 480, 360),
        };

        // Center on screen with cascading offset
        let offset = (self.windows.len() as i32 * 25) % 150;
        let x = ((screen_w as i32 - w as i32) / 2 + offset).max(20);
        let y = ((screen_h as i32 - TASKBAR_HEIGHT as i32 - h as i32) / 2 + offset).max(20);

        let id = self.next_window_id;
        self.next_window_id += 1;

        let window = Window::new(id, app_id, title, icon, x, y, w, h);
        self.windows.push(window);
        self.focus_window(id);
        self.start_menu_open = false;
        self.context_menu_open = false;
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

    pub fn handle_mouse(&mut self, mouse: MouseState) -> Option<(AppId, i32, i32, u32, u32)> {
        let (screen_w, screen_h) = get_dimensions();
        let tb_y = screen_h as i32 - TASKBAR_HEIGHT as i32;
        let now_ms = pit::uptime_ms();
        let mut body_click = None;

        // 1. Right Click on Desktop (Context Menu)
        if mouse.right_button && !self.prev_mouse.right_button {
            if mouse.y < tb_y {
                // Check if right-clicking outside any window
                let on_window = self.windows.iter().rev().any(|w| !w.is_minimized && w.contains_point(mouse.x, mouse.y));
                if !on_window {
                    self.context_menu_open = true;
                    self.context_menu_x = mouse.x.min(screen_w as i32 - 180);
                    self.context_menu_y = mouse.y.min(tb_y - 190);
                    self.start_menu_open = false;
                    self.calendar_open = false;
                    self.prev_mouse = mouse;
                    return None;
                }
            }
        }

        // 2. Dragging Desktop Icon
        if let Some(drag_idx) = self.dragged_icon_idx {
            if mouse.left_button {
                let new_x = (mouse.x - self.drag_offset_x).max(10).min(screen_w as i32 - ICON_CELL_W - 10);
                let new_y = (mouse.y - self.drag_offset_y).max(10).min(tb_y - ICON_CELL_H - 10);
                self.icons[drag_idx].x = new_x;
                self.icons[drag_idx].y = new_y;
            } else {
                // Drop icon and prevent overlapping
                self.dragged_icon_idx = None;
                self.prevent_icon_overlap(drag_idx);
            }
        }

        // 3. Handle Context Menu Click
        if self.context_menu_open && mouse.left_button && !self.prev_mouse.left_button {
            let cm_x = self.context_menu_x;
            let cm_y = self.context_menu_y;
            let cm_w = 175;
            let cm_h = 195;

            if mouse.x >= cm_x && mouse.x <= cm_x + cm_w && mouse.y >= cm_y && mouse.y <= cm_y + cm_h {
                let item_idx = (mouse.y - cm_y - 4) / 26;
                match item_idx {
                    0 => {
                        // Refresh Desktop
                        for icon in &mut self.icons {
                            icon.is_selected = false;
                        }
                    }
                    1 => {
                        // Change Wallpaper
                        self.wallpaper_theme = (self.wallpaper_theme + 1) % 3;
                    }
                    2 => {
                        // Arrange Icons
                        self.arrange_icons();
                    }
                    3 => {
                        // Create Folder
                        let _ = crate::fs::vfs::VFS.lock().create_dir("/home/shrivika/New_Folder");
                    }
                    4 => {
                        // Open Terminal
                        self.open_app(AppId::Terminal);
                    }
                    5 => {
                        // Display Settings
                        self.open_app(AppId::Settings);
                    }
                    6 => {
                        // System Information
                        self.open_app(AppId::SysInfo);
                    }
                    _ => {}
                }
            }
            self.context_menu_open = false;
            self.prev_mouse = mouse;
            return None;
        }

        // 4. Handle Window Titlebar Dragging
        if mouse.left_button {
            let dx = mouse.x - self.prev_mouse.x;
            let dy = mouse.y - self.prev_mouse.y;

            for w in self.windows.iter_mut().rev() {
                if w.is_dragging {
                    w.x += dx;
                    w.y += dy;
                    break;
                }
            }
        } else {
            for w in &mut self.windows {
                w.is_dragging = false;
            }
        }

        // 5. Left Click Down
        if mouse.left_button && !self.prev_mouse.left_button {
            // A. Check Taskbar Click
            if mouse.y >= tb_y {
                self.context_menu_open = false;

                // 1. Start Button: (4..105)
                if mouse.x >= 4 && mouse.x <= 105 {
                    self.start_menu_open = !self.start_menu_open;
                    self.calendar_open = false;
                    self.prev_mouse = mouse;
                    return None;
                }

                // 2. Pinned Quick-Launch App Shortcuts: (112..250)
                // Browser (112..142), Terminal (146..176), File Manager (180..210), Games (214..244)
                if mouse.x >= 112 && mouse.x <= 142 {
                    self.open_app(AppId::Browser);
                    self.prev_mouse = mouse;
                    return None;
                }
                if mouse.x >= 146 && mouse.x <= 176 {
                    self.open_app(AppId::Terminal);
                    self.prev_mouse = mouse;
                    return None;
                }
                if mouse.x >= 180 && mouse.x <= 210 {
                    self.open_app(AppId::FileManager);
                    self.prev_mouse = mouse;
                    return None;
                }
                if mouse.x >= 214 && mouse.x <= 244 {
                    self.open_app(AppId::Games);
                    self.prev_mouse = mouse;
                    return None;
                }

                // 3. System Tray Clicks (Right side)
                let tray_start_x = screen_w as i32 - 270;
                // Wi-Fi Icon Click: opens Network Manager
                if mouse.x >= tray_start_x + 35 && mouse.x <= tray_start_x + 95 {
                    self.open_app(AppId::NetworkManager);
                    self.prev_mouse = mouse;
                    return None;
                }

                // Volume Icon Click: toggles mute
                if mouse.x >= tray_start_x + 100 && mouse.x <= tray_start_x + 125 {
                    self.volume_muted = !self.volume_muted;
                    self.prev_mouse = mouse;
                    return None;
                }

                // Clock/Date Click: toggles Calendar Pop-up Panel
                if mouse.x >= tray_start_x + 130 {
                    self.calendar_open = !self.calendar_open;
                    self.start_menu_open = false;
                    self.prev_mouse = mouse;
                    return None;
                }

                // 4. Running Window Tabs (256..tray_start_x)
                let mut tab_x = 256;
                let tab_w = 135;
                for w in &self.windows {
                    if mouse.x >= tab_x && mouse.x < tab_x + tab_w {
                        if w.is_minimized {
                            self.toggle_minimize(w.id);
                        } else if w.is_active {
                            self.toggle_minimize(w.id);
                        } else {
                            self.focus_window(w.id);
                        }
                        self.prev_mouse = mouse;
                        return None;
                    }
                    tab_x += tab_w + 4;
                }

                self.prev_mouse = mouse;
                return None;
            }

            // B. Check Start Menu Click
            if self.start_menu_open {
                let sm_x = 4;
                let sm_y = tb_y - START_MENU_HEIGHT as i32;
                if mouse.x >= sm_x && mouse.x <= sm_x + START_MENU_WIDTH as i32 && mouse.y >= sm_y && mouse.y < tb_y {
                    self.handle_start_menu_click(mouse.x - sm_x, mouse.y - sm_y);
                    self.prev_mouse = mouse;
                    return None;
                } else {
                    self.start_menu_open = false;
                }
            }

            // C. Check Calendar Popup Click
            if self.calendar_open {
                let cal_w = 230;
                let cal_h = 240;
                let cal_x = screen_w as i32 - cal_w - 8;
                let cal_y = tb_y - cal_h - 4;
                if mouse.x >= cal_x && mouse.x <= cal_x + cal_w && mouse.y >= cal_y && mouse.y < tb_y {
                    // Inside calendar
                    self.prev_mouse = mouse;
                    return None;
                } else {
                    self.calendar_open = false;
                }
            }

            // D. Check Window Interactions (Top to Bottom Z-order)
            let mut clicked_win_id = None;
            for w in self.windows.iter_mut().rev() {
                if !w.is_minimized && w.contains_point(mouse.x, mouse.y) {
                    clicked_win_id = Some(w.id);

                    // Check close button
                    if w.close_button_contains(mouse.x, mouse.y) {
                        let id = w.id;
                        self.close_window(id);
                        self.prev_mouse = mouse;
                        return None;
                    }

                    // Check maximize button
                    if w.maximize_button_contains(mouse.x, mouse.y) {
                        let id = w.id;
                        self.toggle_maximize(id);
                        self.prev_mouse = mouse;
                        return None;
                    }

                    // Check minimize button
                    if w.minimize_button_contains(mouse.x, mouse.y) {
                        let id = w.id;
                        self.toggle_minimize(id);
                        self.prev_mouse = mouse;
                        return None;
                    }

                    // Check titlebar drag
                    if w.titlebar_contains(mouse.x, mouse.y) && !w.is_maximized {
                        w.is_dragging = true;
                    } else if mouse.y >= w.body_y() {
                        let local_x = mouse.x - w.body_x();
                        let local_y = mouse.y - w.body_y();
                        body_click = Some((w.app_id, local_x, local_y, w.body_width(), w.body_height()));
                    }

                    break;
                }
            }

            if let Some(id) = clicked_win_id {
                self.focus_window(id);
                self.prev_mouse = mouse;
                return body_click;
            }

            // E. Check Desktop Icon Click & Selection
            let mut clicked_icon = None;
            for (idx, icon) in self.icons.iter_mut().enumerate() {
                if mouse.x >= icon.x && mouse.x <= icon.x + ICON_CELL_W && mouse.y >= icon.y && mouse.y <= icon.y + ICON_CELL_H {
                    clicked_icon = Some(idx);
                    break;
                }
            }

            if let Some(idx) = clicked_icon {
                // Double click detection
                let is_double_click = self.last_click_icon_idx == Some(idx) && (now_ms - self.last_click_time_ms < 450);
                self.last_click_time_ms = now_ms;
                self.last_click_icon_idx = Some(idx);

                for (i, icon) in self.icons.iter_mut().enumerate() {
                    icon.is_selected = i == idx;
                }

                if is_double_click {
                    // Launch Application
                    if let Some(app_id) = self.icons[idx].app_id {
                        self.open_app(app_id);
                    } else {
                        // Trash
                        self.open_app(AppId::FileManager);
                    }
                } else {
                    // Start dragging icon
                    self.dragged_icon_idx = Some(idx);
                    self.drag_offset_x = mouse.x - self.icons[idx].x;
                    self.drag_offset_y = mouse.y - self.icons[idx].y;
                }
            } else {
                // Clicked on empty desktop: unselect all icons
                for icon in &mut self.icons {
                    icon.is_selected = false;
                }
                self.last_click_icon_idx = None;
            }
        }

        self.prev_mouse = mouse;
        body_click
    }

    fn prevent_icon_overlap(&mut self, moved_idx: usize) {
        let mx = self.icons[moved_idx].x;
        let my = self.icons[moved_idx].y;

        for (i, other) in self.icons.iter_mut().enumerate() {
            if i != moved_idx {
                let dist_x = (mx - other.x).abs();
                let dist_y = (my - other.y).abs();
                if dist_x < 50 && dist_y < 50 {
                    // Offset the moved icon to prevent overlap
                    self.icons[moved_idx].x += 60;
                    if self.icons[moved_idx].x > 600 {
                        self.icons[moved_idx].x = 20;
                        self.icons[moved_idx].y += 75;
                    }
                    break;
                }
            }
        }
    }

    fn handle_start_menu_click(&mut self, local_x: i32, local_y: i32) {
        // App launcher items: y from 68 to 290
        if local_y >= 68 && local_y <= 295 {
            let item_idx = (local_y - 68) / 28;
            match item_idx {
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
            self.start_menu_open = false;
            return;
        }

        // Power Buttons at bottom (320..350)
        if local_y >= 320 && local_y <= 352 {
            if local_x >= 10 && local_x <= 115 {
                crate::power::shutdown();
            } else if local_x >= 140 && local_x <= 245 {
                crate::power::reboot();
            }
        }
    }

    pub fn render_wallpaper(&mut self) {
        let (screen_w, screen_h) = get_dimensions();
        self.init_icons();

        match self.wallpaper_theme {
            1 => {
                // Deep Cobalt Aurora
                draw_gradient_v(0, 0, screen_w as u32, screen_h as u32, Color::rgb(10, 25, 47), Color::rgb(2, 12, 27));
                for y in (0..screen_h).step_by(80) {
                    fill_rect(0, y as i32, screen_w as u32, 1, Color::rgba(6, 182, 212, 20));
                }
            }
            2 => {
                // Midnight Slate (Obsidian Clean)
                draw_gradient_v(0, 0, screen_w as u32, screen_h as u32, Color::rgb(15, 23, 42), Color::rgb(9, 14, 26));
                for x in (0..screen_w).step_by(32) {
                    for y in (0..screen_h).step_by(32) {
                        fill_rect(x as i32, y as i32, 1, 1, Color::rgba(148, 163, 184, 40));
                    }
                }
            }
            _ => {
                // Theme 0: Modern Cyber Grid (Default)
                draw_gradient_v(0, 0, screen_w as u32, screen_h as u32, Color::rgb(8, 12, 22), Color::rgb(3, 5, 11));
                let grid_size = 48;
                for x in (0..screen_w).step_by(grid_size) {
                    fill_rect(x as i32, 0, 1, screen_h as u32, Color::rgba(30, 41, 59, 100));
                }
                for y in (0..screen_h).step_by(grid_size) {
                    fill_rect(0, y as i32, screen_w as u32, 1, Color::rgba(30, 41, 59, 100));
                }
            }
        }

        // Render Desktop Branding Watermark in top-right
        draw_text(screen_w as i32 - 210, 16, "LIFE OS — Shrivika", Color::rgba(255, 255, 255, 90), None);
        draw_text(screen_w as i32 - 210, 32, "Bare-Metal Rust Microkernel", Color::rgba(6, 182, 212, 110), None);

        // Render Desktop Application Icons
        for icon in &self.icons {
            let ix = icon.x;
            let iy = icon.y;

            if icon.is_selected {
                // Glowing selection background
                fill_rect(ix - 4, iy - 4, ICON_CELL_W as u32 + 8, ICON_CELL_H as u32 + 8, Color::rgba(6, 182, 212, 50));
                draw_rect_outline(ix - 4, iy - 4, ICON_CELL_W as u32 + 8, ICON_CELL_H as u32 + 8, 1, Color::rgb(6, 182, 212));
            }

            // Draw large 32x32 icon centered horizontally in 76px cell
            let icon_center_x = ix + (ICON_CELL_W - 32) / 2;
            draw_icon_large(icon_center_x, iy + 4, icon.icon);

            // Draw icon name with soft shadow
            let text_y = iy + 42;
            let name_len = icon.name.len() as i32 * 8;
            let text_x = ix + (ICON_CELL_W - name_len) / 2;

            draw_text(text_x + 1, text_y + 1, icon.name, Color::rgb(0, 0, 0), None); // Shadow
            draw_text(text_x, text_y, icon.name, Color::rgb(241, 245, 249), None);
        }
    }

    pub fn render_context_menu(&self) {
        let x = self.context_menu_x;
        let y = self.context_menu_y;
        let w = 175;
        let h = 195;

        fill_rect(x, y, w, h, Color::rgb(15, 23, 42));
        draw_rect_outline(x, y, w, h, 1, Color::rgb(6, 182, 212));

        let items = [
            "Refresh Desktop",
            "Change Wallpaper",
            "Arrange Icons",
            "Create Folder",
            "Open Terminal",
            "Display Settings",
            "System Info",
        ];

        let mut item_y = y + 4;
        for item in &items {
            draw_text(x + 12, item_y + 6, item, Color::rgb(241, 245, 249), None);
            fill_rect(x + 6, item_y + 24, w - 12, 1, Color::rgb(30, 41, 59));
            item_y += 26;
        }
    }

    pub fn render_taskbar(&self) {
        let (screen_w, screen_h) = get_dimensions();
        let tb_y = screen_h as i32 - TASKBAR_HEIGHT as i32;

        // Taskbar background
        draw_gradient_v(0, tb_y, screen_w as u32, TASKBAR_HEIGHT, Color::rgb(15, 23, 42), Color::rgb(8, 12, 22));
        fill_rect(0, tb_y, screen_w as u32, 1, Color::rgb(51, 65, 85));

        // 1. Start Button (0..105)
        let start_btn_bg = if self.start_menu_open {
            Color::rgb(6, 182, 212)
        } else {
            Color::rgb(30, 41, 59)
        };
        fill_rect(4, tb_y + 4, 102, TASKBAR_HEIGHT - 8, start_btn_bg);
        draw_icon(8, tb_y + 11, AppIcon::LifeOsLogo);
        draw_text(30, tb_y + 11, "Life OS", Color::rgb(255, 255, 255), None);

        // 2. Pinned Quick-Launch App Shortcuts
        let shortcuts = [
            (112, AppIcon::Browser),
            (146, AppIcon::Terminal),
            (180, AppIcon::FileManager),
            (214, AppIcon::Games),
        ];
        for (sx, s_icon) in shortcuts {
            fill_rect(sx, tb_y + 5, 28, TASKBAR_HEIGHT - 10, Color::rgb(24, 33, 47));
            draw_icon(sx + 6, tb_y + 11, s_icon);
        }

        // Divider
        fill_rect(248, tb_y + 6, 1, TASKBAR_HEIGHT - 12, Color::rgb(51, 65, 85));

        // 3. Running Window Tabs (256..)
        let mut tab_x = 256;
        let tab_w = 135;
        for w in &self.windows {
            let tab_bg = if w.is_active && !w.is_minimized {
                Color::rgb(30, 41, 59)
            } else {
                Color::rgb(20, 27, 40)
            };
            fill_rect(tab_x, tb_y + 5, tab_w, TASKBAR_HEIGHT - 10, tab_bg);
            if w.is_active && !w.is_minimized {
                fill_rect(tab_x, tb_y + TASKBAR_HEIGHT as i32 - 3, tab_w, 2, Color::rgb(6, 182, 212));
            }
            draw_icon(tab_x + 4, tb_y + 11, w.icon);
            draw_text_clipped(tab_x + 24, tb_y + 11, tab_w - 28, &w.title, Color::rgb(241, 245, 249), None);
            tab_x += tab_w as i32 + 4;
        }

        // 4. System Tray (Right side)
        let tray_x = screen_w as i32 - 270;

        // Ethernet status (e1000)
        let eth_up = e1000::is_up();
        let eth_color = if eth_up { Color::rgb(34, 197, 94) } else { Color::rgb(239, 68, 68) };
        fill_rect(tray_x + 6, tb_y + 15, 8, 8, eth_color);
        draw_text(tray_x + 18, tb_y + 11, "ETH", Color::rgb(203, 213, 225), None);

        // Wi-Fi Status Indicator
        let (wifi_conn, wifi_ssid, _wifi_sig) = crate::apps::network_manager::get_wifi_tray_status();
        let wifi_color = if wifi_conn {
            Color::rgb(34, 197, 94)
        } else {
            Color::rgb(148, 163, 184)
        };
        draw_icon(tray_x + 50, tb_y + 11, AppIcon::Network);
        let wifi_label = if !wifi_ssid.is_empty() {
            &wifi_ssid[..]
        } else if wifi_conn {
            "Wi-Fi"
        } else {
            "Bridge"
        };
        draw_text_clipped(tray_x + 70, tb_y + 11, 40, wifi_label, wifi_color, None);

        // Volume Icon
        draw_icon(tray_x + 115, tb_y + 11, AppIcon::Volume);
        if self.volume_muted {
            fill_rect(tray_x + 128, tb_y + 13, 3, 3, Color::rgb(239, 68, 68));
        }

        // Clock & Date
        let dt = read_rtc();
        let time_str = format!("{:02}:{:02}:{:02}", dt.hour, dt.minute, dt.second);
        let date_str = format!("{:04}-{:02}-{:02}", dt.year, dt.month, dt.day);

        draw_text(tray_x + 140, tb_y + 4, &time_str, Color::rgb(255, 255, 255), None);
        draw_text(tray_x + 140, tb_y + 20, &date_str, Color::rgb(148, 163, 184), None);

        // Render Calendar Panel if open
        if self.calendar_open {
            self.render_calendar_panel(screen_w as i32, tb_y, &time_str, &date_str, &dt);
        }
    }

    fn render_calendar_panel(&self, screen_w: i32, tb_y: i32, time_str: &str, _date_str: &str, dt: &crate::drivers::rtc::DateTime) {
        let cal_w = 230;
        let cal_h = 240;
        let x = screen_w - cal_w - 8;
        let y = tb_y - cal_h - 4;

        fill_rect(x, y, cal_w as u32, cal_h as u32, Color::rgb(15, 23, 42));
        draw_rect_outline(x, y, cal_w as u32, cal_h as u32, 1, Color::rgb(6, 182, 212));

        // Time header
        fill_rect(x + 1, y + 1, cal_w as u32 - 2, 45, Color::rgb(30, 41, 59));
        draw_text(x + 20, y + 8, time_str, Color::rgb(255, 255, 255), None);
        draw_text(x + 20, y + 26, &format!("November {}, 2026", dt.day), Color::rgb(6, 182, 212), None);

        // Calendar Grid Header (Days of week)
        let days = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];
        let mut dx = x + 12;
        for d in days {
            draw_text(dx, y + 54, d, Color::rgb(148, 163, 184), None);
            dx += 30;
        }
        fill_rect(x + 8, y + 72, cal_w as u32 - 16, 1, Color::rgb(51, 65, 85));

        // Calendar Dates
        let mut row_y = y + 80;
        let mut col_x = x + 12;
        for day in 1..=30 {
            let is_today = day == dt.day;
            if is_today {
                fill_rect(col_x - 4, row_y - 2, 24, 18, Color::rgb(6, 182, 212));
                draw_text(col_x, row_y, &format!("{:2}", day), Color::rgb(255, 255, 255), None);
            } else {
                draw_text(col_x, row_y, &format!("{:2}", day), Color::rgb(203, 213, 225), None);
            }

            col_x += 30;
            if day % 7 == 0 {
                col_x = x + 12;
                row_y += 22;
            }
        }
    }

    pub fn render_start_menu(&self) {
        if !self.start_menu_open {
            return;
        }

        let (_screen_w, screen_h) = get_dimensions();
        let sm_x = 4;
        let sm_y = screen_h as i32 - TASKBAR_HEIGHT as i32 - START_MENU_HEIGHT as i32;

        // Panel background
        fill_rect(sm_x, sm_y, START_MENU_WIDTH, START_MENU_HEIGHT, Color::rgb(15, 23, 42));
        draw_rect_outline(sm_x, sm_y, START_MENU_WIDTH, START_MENU_HEIGHT, 1, Color::rgb(6, 182, 212));

        // Header: User Profile
        fill_rect(sm_x + 1, sm_y + 1, START_MENU_WIDTH - 2, 54, Color::rgb(30, 41, 59));
        fill_rect(sm_x + 12, sm_y + 10, 34, 34, Color::rgb(6, 182, 212));
        draw_text(sm_x + 24, sm_y + 18, "S", Color::rgb(255, 255, 255), None);

        draw_text(sm_x + 54, sm_y + 12, "Shrivika", Color::rgb(255, 255, 255), None);
        draw_text(sm_x + 54, sm_y + 30, "shrivika@life-os", Color::rgb(148, 163, 184), None);

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

        let mut item_y = sm_y + 66;
        for (name, icon) in &items {
            draw_icon(sm_x + 14, item_y + 4, *icon);
            draw_text(sm_x + 38, item_y + 4, name, Color::rgb(241, 245, 249), None);
            item_y += 28;
        }

        // Bottom Power Buttons (Shutdown & Reboot)
        let btn_y = sm_y + START_MENU_HEIGHT as i32 - 36;
        fill_rect(sm_x + 10, btn_y, 115, 26, Color::rgb(239, 68, 68)); // Red Shutdown
        draw_text(sm_x + 32, btn_y + 5, "Shutdown", Color::rgb(255, 255, 255), None);

        fill_rect(sm_x + 140, btn_y, 115, 26, Color::rgb(245, 158, 11)); // Amber Reboot
        draw_text(sm_x + 168, btn_y + 5, "Reboot", Color::rgb(255, 255, 255), None);
    }
}
