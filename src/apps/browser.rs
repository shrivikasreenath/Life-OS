use crate::graphics::canvas::{draw_rect_outline, draw_text, draw_text_clipped, fill_rect};
use crate::graphics::framebuffer::Color;
use crate::gui::window::Window;
use crate::net::stack::{http_get, resolve_domain};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use pc_keyboard::DecodedKey;
use smoltcp::wire::Ipv4Address;
use spin::Mutex;

pub struct BrowserApp {
    url_input: String,
    current_url: String,
    history: Vec<String>,
    history_idx: usize,
    page_title: String,
    page_content: Vec<String>,
    scroll_y: usize,
    is_loading: bool,
    status_text: String,
}

pub static BROWSER: Mutex<BrowserApp> = Mutex::new(BrowserApp::new());

impl BrowserApp {
    pub const fn new() -> Self {
        Self {
            url_input: String::new(),
            current_url: String::new(),
            history: Vec::new(),
            history_idx: 0,
            page_title: String::new(),
            page_content: Vec::new(),
            scroll_y: 0,
            is_loading: false,
            status_text: String::new(),
        }
    }

    pub fn init(&mut self) {
        self.load_welcome_page();
    }

    pub fn load_welcome_page(&mut self) {
        self.url_input = "life://welcome".to_string();
        self.current_url = "life://welcome".to_string();
        self.page_title = "Welcome to Life OS — Shrivika's College Project".to_string();
        self.scroll_y = 0;
        self.is_loading = false;
        self.status_text = "Ready".to_string();

        let mut lines = Vec::new();
        lines.push("# Welcome to Life OS".to_string());
        lines.push("An Autonomous Bare-Metal Rust Operating System".to_string());
        lines.push("Designed and Developed by: Shrivika".to_string());
        lines.push("Architecture: x86_64 | Platform: Oracle VirtualBox & Bare-Metal PC".to_string());
        lines.push("--------------------------------------------------------------------------------".to_string());
        lines.push(String::new());
        lines.push("## Key Technical Achievements:".to_string());
        lines.push("  * 100% Custom Rust Kernel (no_std, no_main, genuinely built from scratch)".to_string());
        lines.push("  * Higher-Half Direct Map Paging (HHDM) with Limine Bootloader Protocol".to_string());
        lines.push("  * Physical Frame Allocator + Dynamic 32MB Kernel Heap".to_string());
        lines.push("  * Hardware Interrupts: GDT, TSS, IDT, 8259 PIC, PIT Timer (100 Hz)".to_string());
        lines.push("  * Intel 82540EM (e1000) Gigabit Ethernet Controller Driver".to_string());
        lines.push("  * Genuine TCP/IP Stack with smoltcp: DHCP, ARP, IPv4, ICMP Ping, DNS, HTTP Client".to_string());
        lines.push("  * Windows Host Wi-Fi Bridge: Scans physical laptop Wi-Fi networks through host helper".to_string());
        lines.push("  * Desktop Window Compositor with double-buffering, icons, and typography".to_string());
        lines.push("  * 4 Playable Bare-Metal Games: Snake, Pong, Tic-Tac-Toe, 2048".to_string());
        lines.push("  * In-Memory Virtual File System (LifeFS)".to_string());
        lines.push("  * ISO footprint: Under 10 MB (Strictly below the 100 MB requirement)".to_string());
        lines.push(String::new());
        lines.push("## Quick Navigation Links:".to_string());
        lines.push("  [1] life://welcome  - This welcome page".to_string());
        lines.push("  [2] life://about    - Academic project information".to_string());
        lines.push("  [3] http://10.0.2.2 - Host NAT gateway".to_string());
        lines.push("  [4] http://example.com - External HTTP Web Page (via VirtualBox NAT)".to_string());
        lines.push(String::new());
        lines.push("Enter any URL in the address bar above and press [ENTER] to navigate!".to_string());

        self.page_content = lines;
    }

    pub fn navigate(&mut self, url: &str) {
        let clean_url = url.trim();
        if clean_url.is_empty() {
            return;
        }

        self.current_url = clean_url.to_string();
        self.url_input = clean_url.to_string();
        self.scroll_y = 0;

        if clean_url == "life://welcome" || clean_url == "about:blank" {
            self.load_welcome_page();
            return;
        }

        if clean_url == "life://about" {
            self.page_title = "About Life OS — System Architecture".to_string();
            self.page_content = vec![
                "# About Life OS".to_string(),
                "Author: Shrivika".to_string(),
                "Degree: Bachelor of Engineering (Computer Science)".to_string(),
                "Kernel: Rust x86_64 (Release Build)".to_string(),
                "Memory: 1024 MB RAM, 32 MB Kernel Heap Pool".to_string(),
                "Networking: Intel PRO/1000 MT Desktop (82540EM) + smoltcp".to_string(),
                "Graphics: 1024x768 Linear Framebuffer (32-bit ARGB)".to_string(),
            ];
            return;
        }

        // HTTP Navigation
        self.is_loading = true;
        self.status_text = format!("Connecting to {}...", clean_url);
        self.page_title = format!("Loading {}...", clean_url);

        // Parse host and path
        let stripped = if clean_url.starts_with("http://") {
            &clean_url[7..]
        } else {
            clean_url
        };

        let (host, path) = if let Some(slash_idx) = stripped.find('/') {
            (&stripped[..slash_idx], &stripped[slash_idx..])
        } else {
            (stripped, "/")
        };

        let ip_res = if let Ok(ip) = host.parse::<Ipv4Address>() {
            Some(ip)
        } else {
            resolve_domain(host)
        };

        match ip_res {
            Some(ip) => {
                self.status_text = format!("Fetching http://{}{} from {}...", host, path, ip);
                match http_get(ip, 80, host, path) {
                    Ok(response) => {
                        self.page_title = format!("{} — Life Browser", host);
                        self.status_text = "Page loaded successfully".to_string();
                        self.parse_and_set_content(&response);
                    }
                    Err(err) => {
                        self.page_title = "Connection Error".to_string();
                        self.status_text = format!("Error: {}", err);
                        self.page_content = vec![
                            "# Connection Error".to_string(),
                            format!("Failed to connect to http://{}{}", host, path),
                            format!("Details: {}", err),
                            "Check that VirtualBox NAT or host network connection is active.".to_string(),
                        ];
                    }
                }
            }
            None => {
                self.page_title = "DNS Error".to_string();
                self.status_text = "DNS resolution failed".to_string();
                self.page_content = vec![
                    "# DNS Resolution Failed".to_string(),
                    format!("Could not resolve domain name: {}", host),
                    "Default DNS server: 10.0.2.3 / 8.8.8.8".to_string(),
                ];
            }
        }

        self.is_loading = false;
    }

    fn parse_and_set_content(&mut self, raw_html: &str) {
        // Strip headers and extract simple text/tags
        let body = if let Some(idx) = raw_html.find("\r\n\r\n") {
            &raw_html[idx + 4..]
        } else if let Some(idx) = raw_html.find("\n\n") {
            &raw_html[idx + 2..]
        } else {
            raw_html
        };

        let mut lines = Vec::new();
        for raw_line in body.lines() {
            let line = raw_line.trim();
            if line.is_empty() {
                continue;
            }
            // Simple tag cleanup
            let mut in_tag = false;
            let mut clean = String::new();
            for ch in line.chars() {
                if ch == '<' {
                    in_tag = true;
                } else if ch == '>' {
                    in_tag = false;
                } else if !in_tag {
                    clean.push(ch);
                }
            }
            let trimmed = clean.trim();
            if !trimmed.is_empty() {
                lines.push(trimmed.to_string());
            }
        }

        if lines.is_empty() {
            lines.push("(Empty HTML document)".to_string());
        }

        self.page_content = lines;
    }

    pub fn handle_key(&mut self, key: DecodedKey) {
        match key {
            DecodedKey::Unicode('\n') => {
                let url = self.url_input.clone();
                self.navigate(&url);
            }
            DecodedKey::Unicode('\u{8}') => {
                self.url_input.pop();
            }
            DecodedKey::Unicode(c) => {
                if c >= ' ' && c <= '~' {
                    self.url_input.push(c);
                }
            }
            _ => {}
        }
    }

    pub fn scroll_down(&mut self) {
        if self.scroll_y + 10 < self.page_content.len() {
            self.scroll_y += 1;
        }
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_y > 0 {
            self.scroll_y -= 1;
        }
    }

    pub fn render(&self, win: &Window) {
        let bx = win.body_x();
        let by = win.body_y();
        let bw = win.body_width();
        let bh = win.body_height();

        // 1. Browser Toolbar (Nav buttons + Address bar)
        let bar_h = 32;
        fill_rect(bx, by, bw, bar_h, Color::rgb(30, 41, 59)); // Slate 800

        // Nav buttons: Back (<), Forward (>), Refresh (R), Home (H)
        let btn_y = by + 5;
        // Back
        fill_rect(bx + 6, btn_y, 22, 22, Color::rgb(51, 65, 85));
        draw_text(bx + 12, btn_y + 3, "<", Color::rgb(241, 245, 249), None);

        // Forward
        fill_rect(bx + 32, btn_y, 22, 22, Color::rgb(51, 65, 85));
        draw_text(bx + 38, btn_y + 3, ">", Color::rgb(241, 245, 249), None);

        // Refresh
        fill_rect(bx + 58, btn_y, 22, 22, Color::rgb(51, 65, 85));
        draw_text(bx + 64, btn_y + 3, "R", Color::rgb(241, 245, 249), None);

        // Home
        fill_rect(bx + 84, btn_y, 22, 22, Color::rgb(6, 182, 212));
        draw_text(bx + 90, btn_y + 3, "H", Color::rgb(255, 255, 255), None);

        // URL Input Bar
        let url_bar_x = bx + 112;
        let url_bar_w = if bw > 180 { bw - 180 } else { 60 };
        fill_rect(url_bar_x, btn_y, url_bar_w, 22, Color::rgb(15, 23, 42)); // Slate 900
        draw_rect_outline(url_bar_x, btn_y, url_bar_w, 22, 1, Color::rgb(71, 85, 105));
        draw_text_clipped(
            url_bar_x + 6,
            btn_y + 3,
            url_bar_w - 12,
            &self.url_input,
            Color::rgb(255, 255, 255),
            None,
        );

        // Go Button
        let go_x = url_bar_x + url_bar_w as i32 + 6;
        fill_rect(go_x, btn_y, 40, 22, Color::rgb(16, 185, 129)); // Emerald Green
        draw_text(go_x + 12, btn_y + 3, "Go", Color::rgb(255, 255, 255), None);

        // 2. Web Content View Area
        let content_y = by + bar_h as i32;
        let content_h = if bh > bar_h + 20 { bh - bar_h - 20 } else { 0 };
        fill_rect(bx, content_y, bw, content_h, Color::rgb(248, 250, 252)); // Crisp light document background

        let line_h = 18;
        let visible_lines = (content_h as usize / line_h).saturating_sub(1);
        let end_line = (self.scroll_y + visible_lines).min(self.page_content.len());

        let mut cur_y = content_y + 10;
        for line in &self.page_content[self.scroll_y..end_line] {
            let (color, font_prefix) = if line.starts_with("# ") {
                (Color::rgb(15, 23, 42), &line[2..]) // Heading 1
            } else if line.starts_with("## ") {
                (Color::rgb(2, 132, 199), &line[3..]) // Heading 2 (Cyan/Blue)
            } else if line.starts_with("  * ") {
                (Color::rgb(51, 65, 85), line.as_str()) // Bullet item
            } else {
                (Color::rgb(30, 41, 59), line.as_str()) // Paragraph text
            };

            draw_text_clipped(bx + 16, cur_y, bw - 32, font_prefix, color, None);
            cur_y += line_h as i32;
        }

        // 3. Status Bar at bottom
        let status_y = by + bh as i32 - 20;
        fill_rect(bx, status_y, bw, 20, Color::rgb(30, 41, 59));
        draw_text_clipped(
            bx + 8,
            status_y + 2,
            bw - 16,
            &format!("Status: {} | {}", self.status_text, self.page_title),
            Color::rgb(148, 163, 184),
            None,
        );
    }
}
