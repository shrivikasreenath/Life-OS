use crate::graphics::canvas::{draw_text, fill_rect};
use crate::graphics::framebuffer::Color;
use crate::graphics::icons::{draw_icon, AppIcon};
use crate::gui::window::Window;
use spin::Mutex;

pub struct AboutApp;
pub static ABOUT: Mutex<AboutApp> = Mutex::new(AboutApp);

impl AboutApp {
    pub fn render(&self, win: &Window) {
        let bx = win.body_x();
        let by = win.body_y();
        let bw = win.body_width();
        let bh = win.body_height();

        fill_rect(bx, by, bw, bh, Color::rgb(15, 23, 42));

        // Header Banner
        fill_rect(bx, by, bw, 60, Color::rgb(30, 41, 59));
        draw_icon(bx + 16, by + 16, AppIcon::LifeOsLogo);

        draw_text(bx + 52, by + 12, "LIFE OS — College Project", Color::rgb(6, 182, 212), None);
        draw_text(bx + 52, by + 32, "Autonomous Bare-Metal Rust Operating System", Color::rgb(241, 245, 249), None);

        let mut y = by + 75;
        let spacing = 20;

        draw_text(bx + 20, y, "Student & Developer : Shrivika", Color::rgb(245, 158, 11), None);
        y += spacing;

        draw_text(bx + 20, y, "System Identity     : shrivika@life-os (whoami -> shrivika)", Color::rgb(34, 197, 94), None);
        y += spacing;

        draw_text(bx + 20, y, "Programming Language: Rust (no_std, no_main, bare-metal)", Color::rgb(226, 232, 240), None);
        y += spacing;

        draw_text(bx + 20, y, "Target Architecture : x86_64 Long Mode (BIOS & UEFI)", Color::rgb(226, 232, 240), None);
        y += spacing;

        draw_text(bx + 20, y, "Virtual Machine     : Oracle VirtualBox & QEMU", Color::rgb(226, 232, 240), None);
        y += spacing + 8;

        draw_text(bx + 20, y, "Engineering Highlights:", Color::rgb(6, 182, 212), None);
        y += spacing;

        let bullets = [
            "  * 100% Custom Kernel: Genuinely independent (Not Linux or Windows)",
            "  * Memory Management: Higher-Half Direct Map Paging + 32MB Heap",
            "  * Hardware Interrupts: GDT, TSS, IDT, PIC 8259, PIT Timer at 100 Hz",
            "  * Genuine Networking: Intel 82540EM (e1000) driver + smoltcp TCP/IP",
            "  * Windows Wi-Fi Bridge: Scans physical host Wi-Fi networks",
            "  * 4 Bare-Metal Games: Snake, Pong, Tic-Tac-Toe, 2048",
            "  * Lightweight Footprint: Bootable ISO size < 10 MB (Limit < 100 MB)",
        ];

        for b in &bullets {
            draw_text(bx + 15, y, b, Color::rgb(203, 213, 225), None);
            y += 18;
        }
    }
}
