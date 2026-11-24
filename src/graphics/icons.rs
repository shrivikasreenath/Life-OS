use crate::graphics::canvas::fill_rect;
use crate::graphics::framebuffer::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppIcon {
    Terminal,
    Browser,
    FileManager,
    Network,
    Games,
    SysInfo,
    Settings,
    About,
    LifeOsLogo,
}

pub fn draw_icon(x: i32, y: i32, icon: AppIcon) {
    match icon {
        AppIcon::Terminal => draw_terminal_icon(x, y),
        AppIcon::Browser => draw_browser_icon(x, y),
        AppIcon::FileManager => draw_file_manager_icon(x, y),
        AppIcon::Network => draw_network_icon(x, y),
        AppIcon::Games => draw_games_icon(x, y),
        AppIcon::SysInfo => draw_sysinfo_icon(x, y),
        AppIcon::Settings => draw_settings_icon(x, y),
        AppIcon::About => draw_about_icon(x, y),
        AppIcon::LifeOsLogo => draw_logo_icon(x, y),
    }
}

fn draw_terminal_icon(x: i32, y: i32) {
    // 16x16 Black monitor with green prompt
    fill_rect(x + 1, y + 1, 14, 14, Color::rgb(30, 41, 59)); // Frame
    fill_rect(x + 2, y + 3, 12, 10, Color::rgb(15, 23, 42)); // Screen
    fill_rect(x + 3, y + 5, 2, 2, Color::rgb(34, 197, 94));  // '>'
    fill_rect(x + 4, y + 6, 2, 2, Color::rgb(34, 197, 94));
    fill_rect(x + 3, y + 7, 2, 2, Color::rgb(34, 197, 94));
    fill_rect(x + 7, y + 8, 4, 1, Color::rgb(34, 197, 94));  // '_'
}

fn draw_browser_icon(x: i32, y: i32) {
    // 16x16 Blue globe / compass
    fill_rect(x + 2, y + 2, 12, 12, Color::rgb(14, 165, 233));
    fill_rect(x + 5, y + 3, 6, 10, Color::rgb(2, 132, 199));
    fill_rect(x + 3, y + 6, 10, 4, Color::rgb(56, 189, 248));
    fill_rect(x + 7, y + 4, 2, 8, Color::rgb(240, 249, 255));
}

fn draw_file_manager_icon(x: i32, y: i32) {
    // 16x16 Amber folder
    fill_rect(x + 2, y + 3, 5, 3, Color::rgb(217, 119, 6)); // Tab
    fill_rect(x + 2, y + 5, 12, 9, Color::rgb(245, 158, 11)); // Body
    fill_rect(x + 3, y + 6, 10, 7, Color::rgb(251, 191, 36));
}

fn draw_network_icon(x: i32, y: i32) {
    // 16x16 Wi-Fi signal bars
    fill_rect(x + 2, y + 10, 2, 4, Color::rgb(16, 185, 129));
    fill_rect(x + 5, y + 8, 2, 6, Color::rgb(16, 185, 129));
    fill_rect(x + 8, y + 5, 2, 9, Color::rgb(16, 185, 129));
    fill_rect(x + 11, y + 2, 2, 12, Color::rgb(16, 185, 129));
}

fn draw_games_icon(x: i32, y: i32) {
    // 16x16 Gamepad / joystick
    fill_rect(x + 2, y + 5, 12, 7, Color::rgb(99, 102, 241));
    // D-pad
    fill_rect(x + 4, y + 7, 3, 1, Color::rgb(255, 255, 255));
    fill_rect(x + 5, y + 6, 1, 3, Color::rgb(255, 255, 255));
    // Buttons
    fill_rect(x + 10, y + 7, 2, 2, Color::rgb(244, 63, 94));
    fill_rect(x + 12, y + 8, 2, 2, Color::rgb(234, 179, 8));
}

fn draw_sysinfo_icon(x: i32, y: i32) {
    // 16x16 Microchip CPU
    fill_rect(x + 3, y + 3, 10, 10, Color::rgb(71, 85, 105));
    fill_rect(x + 5, y + 5, 6, 6, Color::rgb(15, 23, 42));
    // Pins
    fill_rect(x + 5, y + 1, 2, 2, Color::rgb(203, 213, 225));
    fill_rect(x + 9, y + 1, 2, 2, Color::rgb(203, 213, 225));
    fill_rect(x + 5, y + 13, 2, 2, Color::rgb(203, 213, 225));
    fill_rect(x + 9, y + 13, 2, 2, Color::rgb(203, 213, 225));
}

fn draw_settings_icon(x: i32, y: i32) {
    // 16x16 Gear
    fill_rect(x + 4, y + 4, 8, 8, Color::rgb(148, 163, 184));
    fill_rect(x + 7, y + 2, 2, 12, Color::rgb(148, 163, 184));
    fill_rect(x + 2, y + 7, 12, 2, Color::rgb(148, 163, 184));
    fill_rect(x + 6, y + 6, 4, 4, Color::rgb(30, 41, 59)); // Hole
}

fn draw_about_icon(x: i32, y: i32) {
    // 16x16 Info bubble
    fill_rect(x + 2, y + 2, 12, 12, Color::rgb(139, 92, 246));
    fill_rect(x + 7, y + 4, 2, 2, Color::rgb(255, 255, 255)); // Dot
    fill_rect(x + 7, y + 7, 2, 5, Color::rgb(255, 255, 255)); // Stem
}

fn draw_logo_icon(x: i32, y: i32) {
    // Life OS Brand Logo: Cyan glowing atom/diamond
    fill_rect(x + 7, y + 1, 2, 14, Color::rgb(6, 182, 212));
    fill_rect(x + 1, y + 7, 14, 2, Color::rgb(6, 182, 212));
    fill_rect(x + 3, y + 3, 10, 10, Color::rgba(6, 182, 212, 180));
    fill_rect(x + 5, y + 5, 6, 6, Color::rgb(255, 255, 255));
}
