use crate::arch::pit;
use crate::drivers::e1000;
use crate::graphics::canvas::{draw_text, fill_rect};
use crate::graphics::framebuffer::{get_dimensions, Color};
use crate::gui::window::Window;
use crate::mm::frame::FRAME_ALLOCATOR;
use crate::mm::heap;
use crate::net::stack::get_info;
use alloc::format;
use spin::Mutex;

pub struct SysInfoApp;
pub static SYSINFO: Mutex<SysInfoApp> = Mutex::new(SysInfoApp);

impl SysInfoApp {
    pub fn render(&self, win: &Window) {
        let bx = win.body_x();
        let by = win.body_y();
        let bw = win.body_width();
        let bh = win.body_height();

        fill_rect(bx, by, bw, bh, Color::rgb(15, 23, 42));

        let fa = FRAME_ALLOCATOR.lock();
        let total_ram_mb = fa.total_bytes() / (1024 * 1024);
        let usable_ram_mb = fa.usable_bytes() / (1024 * 1024);
        let free_ram_mb = fa.free_bytes() / (1024 * 1024);
        let heap_mb = heap::heap_total_bytes() / (1024 * 1024);
        let (screen_w, screen_h) = get_dimensions();
        let (_rx, _tx) = e1000::get_stats();
        let mac = e1000::get_mac();

        let mut y = by + 16;
        let spacing = 22;

        draw_text(bx + 20, y, "LIFE OS HARDWARE & SYSTEM DASHBOARD", Color::rgb(245, 158, 11), None);
        y += spacing + 4;

        let items = [
            ("Operating System", "Life OS 1.0 (Bare-Metal Rust Kernel)"),
            ("Project Owner", "Shrivika"),
            ("Current User", "shrivika"),
            ("Host Machine", "Lenovo LOQ 15 (Ryzen 7 7435HS, RTX 4050, 24GB)"),
            ("Virtual Platform", "Oracle VirtualBox / QEMU x86_64"),
            ("CPU Architecture", "x86_64 Long Mode (64-bit Paged)"),
        ];

        for (k, v) in &items {
            draw_text(bx + 20, y, k, Color::rgb(148, 163, 184), None);
            draw_text(bx + 180, y, v, Color::rgb(241, 245, 249), None);
            y += spacing;
        }

        y += 8;
        draw_text(bx + 20, y, "--- Memory & Display Subsystems ---", Color::rgb(6, 182, 212), None);
        y += spacing;

        let mem_str = format!("{} MB (Usable: {} MB, Free: {} MB)", total_ram_mb, usable_ram_mb, free_ram_mb);
        draw_text(bx + 20, y, "Physical RAM", Color::rgb(148, 163, 184), None);
        draw_text(bx + 180, y, &mem_str, Color::rgb(241, 245, 249), None);
        y += spacing;

        let heap_str = format!("{} MB Dynamic Pool (linked_list_allocator)", heap_mb);
        draw_text(bx + 20, y, "Kernel Heap", Color::rgb(148, 163, 184), None);
        draw_text(bx + 180, y, &heap_str, Color::rgb(241, 245, 249), None);
        y += spacing;

        let res_str = format!("{}x{} (32-bit ARGB Double-Buffered)", screen_w, screen_h);
        draw_text(bx + 20, y, "Display Framebuffer", Color::rgb(148, 163, 184), None);
        draw_text(bx + 180, y, &res_str, Color::rgb(241, 245, 249), None);
        y += spacing;

        y += 8;
        draw_text(bx + 20, y, "--- Network & Performance ---", Color::rgb(16, 185, 129), None);
        y += spacing;

        let net_ip = if let Some(n) = get_info() { format!("{}", n.ip) } else { "10.0.2.15 (Default)".into() };
        draw_text(bx + 20, y, "IPv4 Address", Color::rgb(148, 163, 184), None);
        draw_text(bx + 180, y, &net_ip, Color::rgb(241, 245, 249), None);
        y += spacing;

        let mac_str = format!("{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}", mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);
        draw_text(bx + 20, y, "e1000 Hardware MAC", Color::rgb(148, 163, 184), None);
        draw_text(bx + 180, y, &mac_str, Color::rgb(241, 245, 249), None);
        y += spacing;

        let uptime_str = format!("{} seconds ({} ticks @ 100 Hz)", pit::uptime_seconds(), pit::get_ticks());
        draw_text(bx + 20, y, "System Uptime", Color::rgb(148, 163, 184), None);
        draw_text(bx + 180, y, &uptime_str, Color::rgb(241, 245, 249), None);
    }
}
