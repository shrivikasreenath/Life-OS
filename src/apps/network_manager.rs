use crate::drivers::e1000;
use crate::graphics::canvas::{draw_rect_outline, draw_text, draw_text_clipped, fill_rect};
use crate::graphics::framebuffer::Color;
use crate::graphics::icons::{draw_icon, AppIcon};
use crate::gui::window::Window;
use crate::net::stack::{get_info, query_host_wifi_bridge};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use spin::Mutex;

#[derive(Debug, Clone)]
pub struct WifiNetwork {
    pub ssid: String,
    pub signal_percent: u8,
    pub security: String,
    pub is_connected: bool,
}

pub struct NetworkManagerApp {
    active_tab: usize, // 0 = Ethernet, 1 = Wi-Fi (Host Bridge)
    wifi_networks: Vec<WifiNetwork>,
    selected_wifi_idx: Option<usize>,
    scanning: bool,
    status_msg: String,
}

pub static NETWORK_MANAGER: Mutex<NetworkManagerApp> = Mutex::new(NetworkManagerApp::new());

impl NetworkManagerApp {
    pub const fn new() -> Self {
        Self {
            active_tab: 0,
            wifi_networks: Vec::new(),
            selected_wifi_idx: None,
            scanning: false,
            status_msg: String::new(),
        }
    }

    pub fn scan_wifi(&mut self) {
        self.scanning = true;
        self.status_msg = "Querying Windows Wi-Fi Bridge on host (10.0.2.2:18888)...".to_string();

        match query_host_wifi_bridge() {
            Ok(response) => {
                self.parse_wifi_response(&response);
                self.status_msg = format!("Discovered {} physical Wi-Fi networks from host", self.wifi_networks.len());
                crate::gui::desktop::DESKTOP.lock().wifi_bridge_connected = true;
            }
            Err(err) => {
                self.status_msg = format!("Host Wi-Fi Bridge: {}", err);
                // Keep existing or show informative notice
                if self.wifi_networks.is_empty() {
                    self.wifi_networks.push(WifiNetwork {
                        ssid: "HostBridge_Waiting".to_string(),
                        signal_percent: 0,
                        security: "Launch START_WIFI_HELPER on Windows host".to_string(),
                        is_connected: false,
                    });
                }
            }
        }
        self.scanning = false;
    }

    fn parse_wifi_response(&mut self, data: &str) {
        // Format expected from Windows host bridge:
        // SSID|SIGNAL%|SECURITY|CONNECTED
        self.wifi_networks.clear();
        for line in data.lines() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 3 {
                let ssid = parts[0].trim().to_string();
                let signal = parts[1].trim().parse::<u8>().unwrap_or(50);
                let security = parts[2].trim().to_string();
                let is_connected = if parts.len() >= 4 {
                    parts[3].trim() == "1" || parts[3].trim().eq_ignore_ascii_case("true")
                } else {
                    false
                };

                if !ssid.is_empty() {
                    self.wifi_networks.push(WifiNetwork {
                        ssid,
                        signal_percent: signal,
                        security,
                        is_connected,
                    });
                }
            }
        }
    }

    pub fn handle_click(&mut self, local_x: i32, local_y: i32) {
        // 1. Tab Bar clicks
        if local_y >= 4 && local_y <= 28 {
            if local_x >= 10 && local_x <= 130 {
                self.active_tab = 0; // Ethernet
            } else if local_x >= 140 && local_x <= 280 {
                self.active_tab = 1; // Wi-Fi Bridge
                if self.wifi_networks.is_empty() {
                    self.scan_wifi();
                }
            }
            return;
        }

        // 2. Wi-Fi tab action buttons
        if self.active_tab == 1 {
            // "Scan Now" button
            if local_x >= 10 && local_x <= 90 && local_y >= 36 && local_y <= 60 {
                self.scan_wifi();
                return;
            }

            // Wi-Fi list items
            let list_y = local_y - 70;
            let item_h = 32;
            if list_y >= 0 {
                let idx = (list_y / item_h) as usize;
                if idx < self.wifi_networks.len() {
                    self.selected_wifi_idx = Some(idx);
                    let net = &self.wifi_networks[idx];
                    self.status_msg = format!("Selected network: {} (Signal: {}%)", net.ssid, net.signal_percent);
                }
            }
        }
    }

    pub fn render(&self, win: &Window) {
        let bx = win.body_x();
        let by = win.body_y();
        let bw = win.body_width();
        let bh = win.body_height();

        // Background
        fill_rect(bx, by, bw, bh, Color::rgb(15, 23, 42));

        // Tabs header
        fill_rect(bx, by, bw, 32, Color::rgb(30, 41, 59));

        // Tab 0: Virtual Ethernet
        let eth_tab_bg = if self.active_tab == 0 { Color::rgb(6, 182, 212) } else { Color::rgb(51, 65, 85) };
        fill_rect(bx + 10, by + 4, 120, 24, eth_tab_bg);
        draw_text(bx + 20, by + 8, "Ethernet (e1000)", Color::rgb(255, 255, 255), None);

        // Tab 1: Physical Wi-Fi Bridge
        let wifi_tab_bg = if self.active_tab == 1 { Color::rgb(6, 182, 212) } else { Color::rgb(51, 65, 85) };
        fill_rect(bx + 140, by + 4, 140, 24, wifi_tab_bg);
        draw_text(bx + 148, by + 8, "Wi-Fi (Host Bridge)", Color::rgb(255, 255, 255), None);

        let content_y = by + 36;

        if self.active_tab == 0 {
            // ETHERNET DASHBOARD
            let (rx, tx) = e1000::get_stats();
            let mac = e1000::get_mac();
            let is_up = e1000::is_up();

            draw_text(bx + 20, content_y + 10, "Virtual Ethernet Adapter Status", Color::rgb(245, 158, 11), None);

            let mut y = content_y + 36;
            let spacing = 22;

            draw_text(bx + 20, y, "Adapter Model   :", Color::rgb(148, 163, 184), None);
            draw_text(bx + 180, y, "Intel PRO/1000 MT Desktop (82540EM)", Color::rgb(241, 245, 249), None);
            y += spacing;

            draw_text(bx + 20, y, "Physical MAC    :", Color::rgb(148, 163, 184), None);
            draw_text(bx + 180, y, &format!("{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}", mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]), Color::rgb(241, 245, 249), None);
            y += spacing;

            draw_text(bx + 20, y, "Hardware Link   :", Color::rgb(148, 163, 184), None);
            let link_str = if is_up { "CONNECTED (1000 Mbps Full Duplex)" } else { "DISCONNECTED" };
            let link_color = if is_up { Color::rgb(34, 197, 94) } else { Color::rgb(239, 68, 68) };
            draw_text(bx + 180, y, link_str, link_color, None);
            y += spacing;

            if let Some(net) = get_info() {
                draw_text(bx + 20, y, "Assigned IPv4   :", Color::rgb(148, 163, 184), None);
                draw_text(bx + 180, y, &format!("{} / {}", net.ip, net.subnet_mask), Color::rgb(6, 182, 212), None);
                y += spacing;

                draw_text(bx + 20, y, "Default Gateway :", Color::rgb(148, 163, 184), None);
                let gw_str = format!("{:?}", net.gateway);
                draw_text(bx + 180, y, &gw_str, Color::rgb(241, 245, 249), None);
                y += spacing;

                draw_text(bx + 20, y, "DHCP Protocol   :", Color::rgb(148, 163, 184), None);
                let dhcp_str = if net.dhcp_configured { "Active & Configured (VirtualBox NAT)" } else { "Pending discovery..." };
                draw_text(bx + 180, y, dhcp_str, Color::rgb(241, 245, 249), None);
                y += spacing;
            }

            draw_text(bx + 20, y, "Packets Transferred :", Color::rgb(148, 163, 184), None);
            draw_text(bx + 180, y, &format!("RX: {} frames | TX: {} frames", rx, tx), Color::rgb(241, 245, 249), None);
        } else {
            // WI-FI HOST BRIDGE
            fill_rect(bx + 10, content_y + 4, 80, 24, Color::rgb(16, 185, 129));
            draw_text(bx + 20, content_y + 8, "Scan Now", Color::rgb(255, 255, 255), None);

            draw_text(
                bx + 105,
                content_y + 8,
                "Nearby Wi-Fi Networks (via Windows Host Bridge)",
                Color::rgb(203, 213, 225),
                None,
            );

            // Table Header
            let list_y = content_y + 34;
            fill_rect(bx + 10, list_y, bw - 20, 20, Color::rgb(24, 33, 47));
            draw_text(bx + 16, list_y + 2, "SSID (Network Name)", Color::rgb(148, 163, 184), None);
            draw_text(bx + 260, list_y + 2, "Signal", Color::rgb(148, 163, 184), None);
            draw_text(bx + 350, list_y + 2, "Security", Color::rgb(148, 163, 184), None);
            draw_text(bx + 450, list_y + 2, "State", Color::rgb(148, 163, 184), None);

            let mut row_y = list_y + 24;
            for (i, net) in self.wifi_networks.iter().enumerate() {
                if row_y + 28 > by + bh as i32 - 24 {
                    break;
                }

                let is_sel = self.selected_wifi_idx == Some(i);
                if is_sel {
                    fill_rect(bx + 10, row_y, bw - 20, 28, Color::rgb(30, 41, 59));
                    draw_rect_outline(bx + 10, row_y, bw - 20, 28, 1, Color::rgb(6, 182, 212));
                }

                draw_icon(bx + 16, row_y + 6, AppIcon::Network);
                draw_text_clipped(bx + 38, row_y + 6, 210, &net.ssid, Color::rgb(255, 255, 255), None);

                // Signal meter bars
                let meter_w = (net.signal_percent as u32 * 60) / 100;
                fill_rect(bx + 260, row_y + 8, 60, 10, Color::rgb(51, 65, 85));
                fill_rect(bx + 260, row_y + 8, meter_w, 10, Color::rgb(16, 185, 129));
                draw_text(bx + 324, row_y + 6, &format!("{}%", net.signal_percent), Color::rgb(148, 163, 184), None);

                // Security
                draw_text_clipped(bx + 350, row_y + 6, 90, &net.security, Color::rgb(203, 213, 225), None);

                // State
                let (st_str, st_color) = if net.is_connected {
                    ("Connected", Color::rgb(34, 197, 94))
                } else {
                    ("Available", Color::rgb(148, 163, 184))
                };
                draw_text(bx + 450, row_y + 6, st_str, st_color, None);

                row_y += 30;
            }
        }

        // Status bar at bottom
        let status_y = by + bh as i32 - 20;
        fill_rect(bx, status_y, bw, 20, Color::rgb(30, 41, 59));
        let status = if self.status_msg.is_empty() {
            "Ready. Note: VirtualBox provides NAT Ethernet; Host Bridge scans physical laptop Wi-Fi.".to_string()
        } else {
            self.status_msg.clone()
        };
        draw_text_clipped(bx + 8, status_y + 2, bw - 16, &status, Color::rgb(148, 163, 184), None);
    }
}
