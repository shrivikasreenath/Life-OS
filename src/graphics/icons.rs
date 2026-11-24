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
    Trash,
    Volume,
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
        AppIcon::Trash => draw_trash_icon(x, y),
        AppIcon::Volume => draw_volume_icon(x, y),
    }
}

pub fn draw_icon_large(x: i32, y: i32, icon: AppIcon) {
    match icon {
        AppIcon::Terminal => draw_terminal_large(x, y),
        AppIcon::Browser => draw_browser_large(x, y),
        AppIcon::FileManager => draw_file_manager_large(x, y),
        AppIcon::Network => draw_network_large(x, y),
        AppIcon::Games => draw_games_large(x, y),
        AppIcon::SysInfo => draw_sysinfo_large(x, y),
        AppIcon::Settings => draw_settings_large(x, y),
        AppIcon::About => draw_about_large(x, y),
        AppIcon::LifeOsLogo => draw_logo_large(x, y),
        AppIcon::Trash => draw_trash_large(x, y),
        AppIcon::Volume => draw_volume_icon(x, y),
    }
}

fn draw_trash_icon(x: i32, y: i32) {
    // 16x16 Trash can
    fill_rect(x + 3, y + 2, 10, 2, Color::rgb(148, 163, 184)); // Lid
    fill_rect(x + 6, y + 1, 4, 1, Color::rgb(148, 163, 184)); // Handle
    fill_rect(x + 4, y + 5, 8, 9, Color::rgb(100, 116, 139)); // Body
    fill_rect(x + 6, y + 7, 1, 5, Color::rgb(71, 85, 105)); // Ribs
    fill_rect(x + 8, y + 7, 1, 5, Color::rgb(71, 85, 105));
}

fn draw_volume_icon(x: i32, y: i32) {
    // 16x16 Speaker
    fill_rect(x + 2, y + 5, 4, 6, Color::rgb(203, 213, 225));
    fill_rect(x + 6, y + 3, 3, 10, Color::rgb(203, 213, 225));
    // Sound waves
    fill_rect(x + 10, y + 5, 2, 6, Color::rgb(6, 182, 212));
    fill_rect(x + 13, y + 3, 2, 10, Color::rgb(6, 182, 212));
}

fn draw_terminal_large(x: i32, y: i32) {
    // 32x32 Modern CRT/Flat Monitor
    fill_rect(x + 2, y + 2, 28, 22, Color::rgb(30, 41, 59));
    fill_rect(x + 4, y + 4, 24, 18, Color::rgb(15, 23, 42));
    // Stand
    fill_rect(x + 13, y + 24, 6, 4, Color::rgb(51, 65, 85));
    fill_rect(x + 9, y + 28, 14, 2, Color::rgb(51, 65, 85));
    // Green Prompt: >_
    fill_rect(x + 6, y + 8, 2, 2, Color::rgb(34, 197, 94));
    fill_rect(x + 8, y + 10, 2, 2, Color::rgb(34, 197, 94));
    fill_rect(x + 6, y + 12, 2, 2, Color::rgb(34, 197, 94));
    fill_rect(x + 12, y + 13, 6, 2, Color::rgb(34, 197, 94));
}

fn draw_browser_large(x: i32, y: i32) {
    // 32x32 Blue Globe / Compass
    fill_rect(x + 4, y + 4, 24, 24, Color::rgb(2, 132, 199));
    fill_rect(x + 8, y + 6, 16, 20, Color::rgb(14, 165, 233));
    fill_rect(x + 6, y + 12, 20, 8, Color::rgb(56, 189, 248));
    // Compass Needle
    fill_rect(x + 15, y + 6, 2, 20, Color::rgb(255, 255, 255));
    fill_rect(x + 11, y + 15, 10, 2, Color::rgb(255, 255, 255));
    fill_rect(x + 18, y + 8, 4, 4, Color::rgb(239, 68, 68)); // Red north pointer
}

fn draw_file_manager_large(x: i32, y: i32) {
    // 32x32 Golden Folder with documents
    fill_rect(x + 4, y + 6, 10, 4, Color::rgb(217, 119, 6)); // Tab
    fill_rect(x + 4, y + 9, 24, 18, Color::rgb(245, 158, 11)); // Back
    // Paper sheet
    fill_rect(x + 8, y + 7, 16, 10, Color::rgb(248, 250, 252));
    fill_rect(x + 10, y + 9, 10, 1, Color::rgb(148, 163, 184));
    fill_rect(x + 10, y + 11, 8, 1, Color::rgb(148, 163, 184));
    // Front pocket
    fill_rect(x + 4, y + 13, 24, 14, Color::rgb(251, 191, 36));
}

fn draw_network_large(x: i32, y: i32) {
    // 32x32 Wi-Fi Signal Radar & Ethernet combo
    fill_rect(x + 5, y + 22, 4, 6, Color::rgb(16, 185, 129));
    fill_rect(x + 11, y + 17, 4, 11, Color::rgb(16, 185, 129));
    fill_rect(x + 17, y + 11, 4, 17, Color::rgb(16, 185, 129));
    fill_rect(x + 23, y + 5, 4, 23, Color::rgb(16, 185, 129));
    // Base dot
    fill_rect(x + 4, y + 26, 24, 2, Color::rgb(6, 182, 212));
}

fn draw_games_large(x: i32, y: i32) {
    // 32x32 Retro Arcade Controller
    fill_rect(x + 3, y + 8, 26, 16, Color::rgb(79, 70, 229));
    fill_rect(x + 5, y + 10, 22, 12, Color::rgb(99, 102, 241));
    // D-Pad (Left)
    fill_rect(x + 7, y + 13, 6, 2, Color::rgb(255, 255, 255));
    fill_rect(x + 9, y + 11, 2, 6, Color::rgb(255, 255, 255));
    // Action Buttons (Right)
    fill_rect(x + 21, y + 11, 3, 3, Color::rgb(244, 63, 94));
    fill_rect(x + 18, y + 14, 3, 3, Color::rgb(234, 179, 8));
    fill_rect(x + 24, y + 14, 3, 3, Color::rgb(34, 197, 94));
    fill_rect(x + 21, y + 17, 3, 3, Color::rgb(6, 182, 212));
}

fn draw_sysinfo_large(x: i32, y: i32) {
    // 32x32 Processor Chip
    fill_rect(x + 5, y + 5, 22, 22, Color::rgb(71, 85, 105));
    fill_rect(x + 9, y + 9, 14, 14, Color::rgb(15, 23, 42));
    // Glowing Core
    fill_rect(x + 13, y + 13, 6, 6, Color::rgb(6, 182, 212));
    // Gold Pins
    for p in (6..26).step_by(4) {
        fill_rect(x + p, y + 2, 2, 3, Color::rgb(245, 158, 11));
        fill_rect(x + p, y + 27, 2, 3, Color::rgb(245, 158, 11));
        fill_rect(x + 2, y + p, 3, 2, Color::rgb(245, 158, 11));
        fill_rect(x + 27, y + p, 3, 2, Color::rgb(245, 158, 11));
    }
}

fn draw_settings_large(x: i32, y: i32) {
    // 32x32 Precision Gear
    fill_rect(x + 8, y + 8, 16, 16, Color::rgb(148, 163, 184));
    fill_rect(x + 13, y + 3, 6, 26, Color::rgb(148, 163, 184));
    fill_rect(x + 3, y + 13, 26, 6, Color::rgb(148, 163, 184));
    fill_rect(x + 6, y + 6, 20, 20, Color::rgb(100, 116, 139));
    // Center Hole
    fill_rect(x + 12, y + 12, 8, 8, Color::rgb(15, 23, 42));
    fill_rect(x + 14, y + 14, 4, 4, Color::rgb(6, 182, 212));
}

fn draw_about_large(x: i32, y: i32) {
    // 32x32 Information Medal
    fill_rect(x + 4, y + 4, 24, 24, Color::rgb(139, 92, 246));
    fill_rect(x + 6, y + 6, 20, 20, Color::rgb(168, 85, 247));
    fill_rect(x + 14, y + 8, 4, 4, Color::rgb(255, 255, 255));
    fill_rect(x + 14, y + 14, 4, 10, Color::rgb(255, 255, 255));
}

fn draw_trash_large(x: i32, y: i32) {
    // 32x32 Metallic Trash Can
    fill_rect(x + 6, y + 5, 20, 3, Color::rgb(148, 163, 184)); // Lid
    fill_rect(x + 12, y + 2, 8, 3, Color::rgb(148, 163, 184)); // Handle
    fill_rect(x + 8, y + 9, 16, 18, Color::rgb(100, 116, 139)); // Body
    fill_rect(x + 11, y + 12, 2, 12, Color::rgb(71, 85, 105)); // Ribs
    fill_rect(x + 15, y + 12, 2, 12, Color::rgb(71, 85, 105));
    fill_rect(x + 19, y + 12, 2, 12, Color::rgb(71, 85, 105));
    fill_rect(x + 7, y + 27, 18, 2, Color::rgb(71, 85, 105)); // Base
}

fn draw_logo_large(x: i32, y: i32) {
    // 32x32 Life OS Glowing Atom
    fill_rect(x + 14, y + 2, 4, 28, Color::rgb(6, 182, 212));
    fill_rect(x + 2, y + 14, 28, 4, Color::rgb(6, 182, 212));
    fill_rect(x + 6, y + 6, 20, 20, Color::rgba(6, 182, 212, 180));
    fill_rect(x + 10, y + 10, 12, 12, Color::rgb(255, 255, 255));
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
