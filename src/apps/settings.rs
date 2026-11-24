use crate::graphics::canvas::{draw_text, fill_rect};
use crate::graphics::framebuffer::{get_dimensions, Color};
use crate::gui::window::Window;
use alloc::format;
use spin::Mutex;

pub struct SettingsApp;
pub static SETTINGS: Mutex<SettingsApp> = Mutex::new(SettingsApp);

impl SettingsApp {
    pub fn handle_click(&self, local_x: i32, local_y: i32) {
        // Shutdown button at y = 260
        if local_y >= 260 && local_y <= 290 {
            if local_x >= 20 && local_x <= 130 {
                crate::power::shutdown();
            } else if local_x >= 150 && local_x <= 260 {
                crate::power::reboot();
            }
        }
    }

    pub fn render(&self, win: &Window) {
        let bx = win.body_x();
        let by = win.body_y();
        let bw = win.body_width();
        let bh = win.body_height();

        fill_rect(bx, by, bw, bh, Color::rgb(15, 23, 42));

        let (screen_w, screen_h) = get_dimensions();
        let mut y = by + 16;
        let spacing = 26;

        draw_text(bx + 20, y, "SYSTEM SETTINGS", Color::rgb(245, 158, 11), None);
        y += spacing + 4;

        // Display
        draw_text(bx + 20, y, "Display Resolution :", Color::rgb(148, 163, 184), None);
        draw_text(bx + 190, y, &format!("{} x {} (GOP/VESA 32bpp)", screen_w, screen_h), Color::rgb(241, 245, 249), None);
        y += spacing;

        // UI Theme
        draw_text(bx + 20, y, "Desktop Theme      :", Color::rgb(148, 163, 184), None);
        draw_text(bx + 190, y, "Dark Modern Slate & Cyan Accent", Color::rgb(6, 182, 212), None);
        y += spacing;

        // Date/Time
        draw_text(bx + 20, y, "Clock & Timezone   :", Color::rgb(148, 163, 184), None);
        draw_text(bx + 190, y, "CMOS Real-Time Clock (RTC 24h)", Color::rgb(241, 245, 249), None);
        y += spacing;

        // Network Mode
        draw_text(bx + 20, y, "Network Interface  :", Color::rgb(148, 163, 184), None);
        draw_text(bx + 190, y, "VirtualBox NAT + Windows Host Wi-Fi Bridge", Color::rgb(16, 185, 129), None);
        y += spacing;

        // Storage Mode
        draw_text(bx + 20, y, "Filesystem Subsystem:", Color::rgb(148, 163, 184), None);
        draw_text(bx + 190, y, "LifeFS Virtual Memory Tree", Color::rgb(241, 245, 249), None);
        y += spacing + 10;

        // Power actions
        draw_text(bx + 20, y, "System Power Management", Color::rgb(245, 158, 11), None);
        y += 24;

        fill_rect(bx + 20, y, 110, 30, Color::rgb(239, 68, 68)); // Red Shutdown
        draw_text(bx + 35, y + 7, "Shutdown", Color::rgb(255, 255, 255), None);

        fill_rect(bx + 150, y, 110, 30, Color::rgb(245, 158, 11)); // Amber Reboot
        draw_text(bx + 175, y + 7, "Reboot", Color::rgb(255, 255, 255), None);
    }
}
