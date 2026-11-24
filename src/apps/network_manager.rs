use crate::arch::pit;
use crate::drivers::e1000;
use crate::graphics::canvas::{draw_rect_outline, draw_text, draw_text_clipped, fill_rect};
use crate::graphics::framebuffer::Color;
use crate::graphics::icons::{draw_icon, AppIcon};
use crate::gui::window::Window;
use crate::net::stack::{
    cancel_host_wifi_command, check_host_wifi_response, get_info, start_host_wifi_command,
    SocketHandle,
};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use pc_keyboard::{DecodedKey, KeyCode};
use spin::Mutex;

#[derive(Debug, Clone)]
pub struct WifiNetwork {
    pub ssid: String,
    pub signal_percent: u8,
    pub security: String,
    pub band: String,
    pub channel: String,
    pub is_connected: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiOpState {
    Idle,
    Scanning {
        handle: SocketHandle,
        start_ms: u64,
        anim_frame: u8,
    },
    Connecting {
        handle: SocketHandle,
        start_ms: u64,
        anim_frame: u8,
    },
    Disconnecting {
        handle: SocketHandle,
        start_ms: u64,
    },
}

pub struct NetworkManagerApp {
    pub active_tab: usize, // 0 = Ethernet, 1 = Wi-Fi (Host Bridge)
    pub wifi_networks: Vec<WifiNetwork>,
    pub selected_wifi_idx: Option<usize>,
    pub status_msg: String,
    pub password_input: String,
    pub password_active: bool,
    pub connected_ssid: String,
    pub connecting_target_ssid: String,
    pub op_state: WifiOpState,
    pub last_update_ms: u64,
}

pub static NETWORK_MANAGER: Mutex<NetworkManagerApp> = Mutex::new(NetworkManagerApp::new());

pub fn get_wifi_tray_status() -> (bool, String, u8) {
    if let Some(mgr) = NETWORK_MANAGER.try_lock() {
        let is_connected = !mgr.connected_ssid.is_empty();
        let ssid = mgr.connected_ssid.clone();
        let sig = mgr
            .wifi_networks
            .iter()
            .find(|n| n.ssid == mgr.connected_ssid)
            .map(|n| n.signal_percent)
            .unwrap_or(85);
        (is_connected, ssid, sig)
    } else {
        (false, String::new(), 0)
    }
}

impl NetworkManagerApp {
    pub const fn new() -> Self {
        Self {
            active_tab: 0,
            wifi_networks: Vec::new(),
            selected_wifi_idx: None,
            status_msg: String::new(),
            password_input: String::new(),
            password_active: false,
            connected_ssid: String::new(),
            connecting_target_ssid: String::new(),
            op_state: WifiOpState::Idle,
            last_update_ms: 0,
        }
    }

    pub fn is_scanning(&self) -> bool {
        matches!(self.op_state, WifiOpState::Scanning { .. })
    }

    pub fn is_connecting(&self) -> bool {
        matches!(self.op_state, WifiOpState::Connecting { .. })
    }

    pub fn scan_wifi(&mut self) {
        if self.op_state != WifiOpState::Idle {
            return;
        }

        match start_host_wifi_command("SCAN") {
            Ok(handle) => {
                self.op_state = WifiOpState::Scanning {
                    handle,
                    start_ms: pit::uptime_ms(),
                    anim_frame: 0,
                };
                self.status_msg =
                    "Scanning real physical Wi-Fi networks via Windows host... [/]".to_string();
            }
            Err(err) => {
                self.status_msg = format!("Host Wi-Fi Bridge: {}", err);
                if self.wifi_networks.is_empty() {
                    self.wifi_networks.push(WifiNetwork {
                        ssid: "HostBridge_Waiting".to_string(),
                        signal_percent: 0,
                        security: "Launch START_WIFI_HELPER on Windows host".to_string(),
                        band: "2.4 GHz".to_string(),
                        channel: "-".to_string(),
                        is_connected: false,
                    });
                }
            }
        }
    }

    pub fn connect_selected(&mut self) {
        if self.op_state != WifiOpState::Idle {
            return;
        }

        let idx = match self.selected_wifi_idx {
            Some(i) if i < self.wifi_networks.len() => i,
            _ => {
                self.status_msg = "Please select a Wi-Fi network from the list first".to_string();
                return;
            }
        };

        let target_ssid = self.wifi_networks[idx].ssid.clone();
        if target_ssid == "HostBridge_Waiting" {
            self.status_msg = "Cannot connect to placeholder. Please run START_WIFI_HELPER on host and click Scan Now.".to_string();
            return;
        }

        let cmd = format!("CONNECT {}|{}", target_ssid, self.password_input);
        match start_host_wifi_command(&cmd) {
            Ok(handle) => {
                self.connecting_target_ssid = target_ssid.clone();
                self.op_state = WifiOpState::Connecting {
                    handle,
                    start_ms: pit::uptime_ms(),
                    anim_frame: 0,
                };
                self.status_msg =
                    format!("Connecting to '{}' via Windows WLAN API... [/]", target_ssid);
            }
            Err(err) => {
                self.status_msg = format!("Connection error: {}", err);
            }
        }
    }

    pub fn disconnect_selected(&mut self) {
        if self.op_state != WifiOpState::Idle {
            return;
        }

        match start_host_wifi_command("DISCONNECT") {
            Ok(handle) => {
                self.op_state = WifiOpState::Disconnecting {
                    handle,
                    start_ms: pit::uptime_ms(),
                };
                self.status_msg = "Disconnecting via Windows WLAN API...".to_string();
            }
            Err(err) => {
                self.status_msg = format!("Disconnect error: {}", err);
            }
        }
    }

    pub fn update(&mut self) {
        let now_ms = pit::uptime_ms();
        if now_ms - self.last_update_ms < 30 {
            return;
        }
        self.last_update_ms = now_ms;

        match self.op_state {
            WifiOpState::Idle => {}
            WifiOpState::Scanning {
                handle,
                start_ms,
                ref mut anim_frame,
            } => match check_host_wifi_response(handle) {
                Ok(Some(response)) => {
                    self.op_state = WifiOpState::Idle;
                    self.parse_wifi_response(&response);
                    self.status_msg = format!(
                        "Discovered {} physical Wi-Fi network(s) from Windows host",
                        self.wifi_networks.len()
                    );
                    if let Some(conn) = self.wifi_networks.iter().find(|n| n.is_connected) {
                        self.connected_ssid = conn.ssid.clone();
                    }
                }
                Ok(None) => {
                    if now_ms - start_ms > 4000 {
                        cancel_host_wifi_command(handle);
                        self.op_state = WifiOpState::Idle;
                        self.status_msg = "Scan timeout: Windows Wi-Fi Bridge did not respond. Check START_WIFI_HELPER.".to_string();
                        if self.wifi_networks.is_empty() {
                            self.wifi_networks.push(WifiNetwork {
                                ssid: "HostBridge_Waiting".to_string(),
                                signal_percent: 0,
                                security: "Launch START_WIFI_HELPER on Windows host".to_string(),
                                band: "2.4 GHz".to_string(),
                                channel: "-".to_string(),
                                is_connected: false,
                            });
                        }
                    } else {
                        *anim_frame = (*anim_frame + 1) % 4;
                        let spinner = match *anim_frame {
                            0 => "[|]",
                            1 => "[/]",
                            2 => "[-]",
                            _ => "[\\]",
                        };
                        self.status_msg = format!(
                            "Scanning physical Wi-Fi networks via Windows host {} ...",
                            spinner
                        );
                    }
                }
                Err(err) => {
                    cancel_host_wifi_command(handle);
                    self.op_state = WifiOpState::Idle;
                    self.status_msg = format!("Host Wi-Fi Bridge error: {}", err);
                }
            },
            WifiOpState::Connecting {
                handle,
                start_ms,
                ref mut anim_frame,
            } => match check_host_wifi_response(handle) {
                Ok(Some(response)) => {
                    self.op_state = WifiOpState::Idle;
                    self.status_msg = response.clone();
                    if response.starts_with("SUCCESS") {
                        let target = self.connecting_target_ssid.clone();
                        for net in self.wifi_networks.iter_mut() {
                            net.is_connected = net.ssid == target;
                        }
                        self.connected_ssid = target;
                        self.password_input.clear();
                        self.password_active = false;
                    }
                }
                Ok(None) => {
                    if now_ms - start_ms > 7500 {
                        cancel_host_wifi_command(handle);
                        self.op_state = WifiOpState::Idle;
                        self.status_msg = format!(
                            "Connection to '{}' timed out. Check password or signal.",
                            self.connecting_target_ssid
                        );
                    } else {
                        *anim_frame = (*anim_frame + 1) % 4;
                        let spinner = match *anim_frame {
                            0 => "[|]",
                            1 => "[/]",
                            2 => "[-]",
                            _ => "[\\]",
                        };
                        self.status_msg = format!(
                            "Connecting to '{}' via Windows WLAN API {} ...",
                            self.connecting_target_ssid, spinner
                        );
                    }
                }
                Err(err) => {
                    cancel_host_wifi_command(handle);
                    self.op_state = WifiOpState::Idle;
                    self.status_msg = format!("Connection error: {}", err);
                }
            },
            WifiOpState::Disconnecting { handle, start_ms } => {
                match check_host_wifi_response(handle) {
                    Ok(Some(response)) => {
                        self.op_state = WifiOpState::Idle;
                        self.status_msg = response;
                        for net in self.wifi_networks.iter_mut() {
                            net.is_connected = false;
                        }
                        self.connected_ssid.clear();
                    }
                    Ok(None) => {
                        if now_ms - start_ms > 3500 {
                            cancel_host_wifi_command(handle);
                            self.op_state = WifiOpState::Idle;
                            self.status_msg = "Disconnected from Wi-Fi".to_string();
                            self.connected_ssid.clear();
                            for net in self.wifi_networks.iter_mut() {
                                net.is_connected = false;
                            }
                        }
                    }
                    Err(err) => {
                        cancel_host_wifi_command(handle);
                        self.op_state = WifiOpState::Idle;
                        self.status_msg = format!("Disconnect error: {}", err);
                    }
                }
            }
        }
    }

    fn parse_wifi_response(&mut self, data: &str) {
        if data.trim().starts_with("NO_NETWORKS") {
            self.wifi_networks.clear();
            self.status_msg = "No physical Wi-Fi networks found by Windows adapter.".to_string();
            return;
        }

        // Format from Windows host bridge:
        // SSID|SIGNAL%|SECURITY|BAND|CHANNEL|CONNECTED
        // or fallback: SSID|SIGNAL%|SECURITY|CONNECTED
        self.wifi_networks.clear();
        for line in data.lines() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 3 {
                let ssid = parts[0].trim().to_string();
                let signal = parts[1].trim().parse::<u8>().unwrap_or(50);
                let security = parts[2].trim().to_string();

                let (band, channel, is_connected) = if parts.len() >= 6 {
                    let b = parts[3].trim().to_string();
                    let ch = parts[4].trim().to_string();
                    let conn = parts[5].trim() == "1" || parts[5].trim().eq_ignore_ascii_case("true");
                    (b, ch, conn)
                } else if parts.len() == 4 {
                    let conn = parts[3].trim() == "1" || parts[3].trim().eq_ignore_ascii_case("true");
                    ("2.4 GHz".to_string(), "6".to_string(), conn)
                } else {
                    ("2.4 GHz".to_string(), "6".to_string(), false)
                };

                if !ssid.is_empty() {
                    let is_conn = is_connected;
                    self.wifi_networks.push(WifiNetwork {
                        ssid: ssid.clone(),
                        signal_percent: signal,
                        security,
                        band,
                        channel,
                        is_connected: is_conn,
                    });
                    if is_conn {
                        self.connected_ssid = ssid;
                    }
                }
            }
        }
    }

    pub fn handle_key(&mut self, key: DecodedKey) {
        if self.password_active {
            match key {
                DecodedKey::Unicode('\n') | DecodedKey::Unicode('\r') => {
                    self.connect_selected();
                }
                DecodedKey::Unicode('\x08') | DecodedKey::RawKey(KeyCode::Backspace) => {
                    self.password_input.pop();
                }
                DecodedKey::RawKey(KeyCode::Escape) => {
                    self.password_active = false;
                }
                DecodedKey::Unicode(c) if !c.is_control() => {
                    if self.password_input.len() < 64 {
                        self.password_input.push(c);
                    }
                }
                _ => {}
            }
            return;
        }

        // Global keys for Network Manager when not entering password:
        match key {
            DecodedKey::RawKey(KeyCode::Tab) | DecodedKey::Unicode('\t') => {
                self.active_tab = 1 - self.active_tab;
                if self.active_tab == 1 && self.wifi_networks.is_empty() {
                    self.scan_wifi();
                }
            }
            DecodedKey::Unicode('w') | DecodedKey::Unicode('W') => {
                self.active_tab = 1;
                if self.wifi_networks.is_empty() {
                    self.scan_wifi();
                }
            }
            DecodedKey::Unicode('e') | DecodedKey::Unicode('E') => {
                self.active_tab = 0;
            }
            DecodedKey::Unicode('s') | DecodedKey::Unicode('S') | DecodedKey::Unicode('r') | DecodedKey::Unicode('R') => {
                if self.active_tab == 1 {
                    self.scan_wifi();
                }
            }
            DecodedKey::RawKey(KeyCode::ArrowDown) | DecodedKey::Unicode('j') => {
                if self.active_tab == 1 && !self.wifi_networks.is_empty() {
                    let next = match self.selected_wifi_idx {
                        Some(i) if i + 1 < self.wifi_networks.len() => i + 1,
                        Some(_) => 0,
                        None => 0,
                    };
                    self.selected_wifi_idx = Some(next);
                }
            }
            DecodedKey::RawKey(KeyCode::ArrowUp) | DecodedKey::Unicode('k') => {
                if self.active_tab == 1 && !self.wifi_networks.is_empty() {
                    let prev = match self.selected_wifi_idx {
                        Some(i) if i > 0 => i - 1,
                        Some(_) => self.wifi_networks.len().saturating_sub(1),
                        None => 0,
                    };
                    self.selected_wifi_idx = Some(prev);
                }
            }
            DecodedKey::Unicode('p') | DecodedKey::Unicode('P') => {
                if self.active_tab == 1 {
                    self.password_active = true;
                }
            }
            DecodedKey::Unicode('c') | DecodedKey::Unicode('C') | DecodedKey::Unicode('\n') => {
                if self.active_tab == 1 {
                    self.connect_selected();
                }
            }
            DecodedKey::Unicode('d') | DecodedKey::Unicode('D') => {
                if self.active_tab == 1 {
                    self.disconnect_selected();
                }
            }
            _ => {}
        }
    }

    pub fn handle_click(&mut self, local_x: i32, local_y: i32) {
        // 1. Tab Bar clicks
        if local_y >= 4 && local_y <= 28 {
            if local_x >= 10 && local_x <= 130 {
                self.active_tab = 0; // Ethernet
            } else if local_x >= 140 && local_x <= 290 {
                self.active_tab = 1; // Wi-Fi Bridge
                if self.wifi_networks.is_empty() {
                    self.scan_wifi();
                }
            }
            return;
        }

        // 2. Wi-Fi tab action buttons
        if self.active_tab == 1 {
            // "Scan Now" button: (10..95, 38..64)
            if local_x >= 10 && local_x <= 95 && local_y >= 38 && local_y <= 64 {
                self.scan_wifi();
                return;
            }

            // "Connect" button: (325..405, 286..316)
            if local_x >= 325 && local_x <= 405 && local_y >= 286 && local_y <= 316 {
                self.connect_selected();
                return;
            }

            // "Disconnect" button: (415..510, 286..316)
            if local_x >= 415 && local_x <= 510 && local_y >= 286 && local_y <= 316 {
                self.disconnect_selected();
                return;
            }

            // Password input box: (90..315, 286..316)
            if local_x >= 90 && local_x <= 315 && local_y >= 286 && local_y <= 316 {
                self.password_active = true;
                return;
            } else if local_y >= 286 && local_y <= 316 {
                self.password_active = false;
            }

            // Wi-Fi list items: starting at row_y = 92, 28px per row
            let item_start_y = 92;
            let row_h = 28;
            if local_y >= item_start_y {
                let idx = ((local_y - item_start_y) / row_h) as usize;
                if idx < self.wifi_networks.len() {
                    self.selected_wifi_idx = Some(idx);
                    let net = &self.wifi_networks[idx];
                    self.status_msg = format!(
                        "Selected: '{}' (Signal: {}%, Band: {}, Security: {})",
                        net.ssid, net.signal_percent, net.band, net.security
                    );
                    self.password_active = false;
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
        let eth_tab_bg = if self.active_tab == 0 {
            Color::rgb(6, 182, 212)
        } else {
            Color::rgb(51, 65, 85)
        };
        fill_rect(bx + 10, by + 4, 120, 24, eth_tab_bg);
        draw_text(bx + 20, by + 8, "Ethernet (e1000)", Color::rgb(255, 255, 255), None);

        // Tab 1: Physical Wi-Fi Bridge
        let wifi_tab_bg = if self.active_tab == 1 {
            Color::rgb(6, 182, 212)
        } else {
            Color::rgb(51, 65, 85)
        };
        fill_rect(bx + 140, by + 4, 150, 24, wifi_tab_bg);
        draw_text(bx + 148, by + 8, "Wi-Fi (Host Bridge)", Color::rgb(255, 255, 255), None);

        let content_y = by + 36;

        if self.active_tab == 0 {
            // ETHERNET DASHBOARD
            let (rx, tx) = e1000::get_stats();
            let mac = e1000::get_mac();
            let is_up = e1000::is_up();

            draw_text(
                bx + 20,
                content_y + 10,
                "Virtual Ethernet Adapter Status (Intel e1000 NAT)",
                Color::rgb(245, 158, 11),
                None,
            );

            let mut y = content_y + 36;
            let spacing = 22;

            draw_text(bx + 20, y, "Adapter Model   :", Color::rgb(148, 163, 184), None);
            draw_text(
                bx + 180,
                y,
                "Intel PRO/1000 MT Desktop (82540EM)",
                Color::rgb(241, 245, 249),
                None,
            );
            y += spacing;

            draw_text(bx + 20, y, "Physical MAC    :", Color::rgb(148, 163, 184), None);
            draw_text(
                bx + 180,
                y,
                &format!(
                    "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                    mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
                ),
                Color::rgb(241, 245, 249),
                None,
            );
            y += spacing;

            draw_text(bx + 20, y, "Hardware Link   :", Color::rgb(148, 163, 184), None);
            let link_str = if is_up {
                "CONNECTED (1000 Mbps Full Duplex)"
            } else {
                "DISCONNECTED"
            };
            let link_color = if is_up {
                Color::rgb(34, 197, 94)
            } else {
                Color::rgb(239, 68, 68)
            };
            draw_text(bx + 180, y, link_str, link_color, None);
            y += spacing;

            if let Some(net) = get_info() {
                draw_text(bx + 20, y, "Assigned IPv4   :", Color::rgb(148, 163, 184), None);
                draw_text(
                    bx + 180,
                    y,
                    &format!("{} / {}", net.ip, net.subnet_mask),
                    Color::rgb(6, 182, 212),
                    None,
                );
                y += spacing;

                draw_text(bx + 20, y, "Default Gateway :", Color::rgb(148, 163, 184), None);
                let gw_str = format!("{:?}", net.gateway);
                draw_text(bx + 180, y, &gw_str, Color::rgb(241, 245, 249), None);
                y += spacing;

                draw_text(bx + 20, y, "DHCP Protocol   :", Color::rgb(148, 163, 184), None);
                let dhcp_str = if net.dhcp_configured {
                    "Active & Configured (VirtualBox NAT)"
                } else {
                    "Static / Configured via NAT fallback"
                };
                draw_text(bx + 180, y, dhcp_str, Color::rgb(241, 245, 249), None);
                y += spacing;
            }

            draw_text(bx + 20, y, "Packets Transferred :", Color::rgb(148, 163, 184), None);
            draw_text(
                bx + 180,
                y,
                &format!("RX: {} frames | TX: {} frames", rx, tx),
                Color::rgb(241, 245, 249),
                None,
            );
        } else {
            // WI-FI HOST BRIDGE TAB
            let scan_btn_color = if self.is_scanning() {
                Color::rgb(245, 158, 11) // Amber
            } else {
                Color::rgb(16, 185, 129) // Emerald
            };
            let scan_btn_text = if self.is_scanning() { "Scanning..." } else { "Scan Now" };
            fill_rect(bx + 10, content_y + 4, 85, 24, scan_btn_color);
            let scan_text_x = if self.is_scanning() { bx + 12 } else { bx + 18 };
            draw_text(scan_text_x, content_y + 8, scan_btn_text, Color::rgb(255, 255, 255), None);

            draw_text(
                bx + 110,
                content_y + 8,
                "Real Physical Wi-Fi Networks (Host Bridge 10.0.2.2:18888)",
                Color::rgb(203, 213, 225),
                None,
            );

            // Table Header
            let list_y = content_y + 34;
            fill_rect(bx + 10, list_y, bw - 20, 20, Color::rgb(24, 33, 47));
            draw_text(bx + 16, list_y + 2, "SSID (Network Name)", Color::rgb(148, 163, 184), None);
            draw_text(bx + 230, list_y + 2, "Signal", Color::rgb(148, 163, 184), None);
            draw_text(bx + 310, list_y + 2, "Band / Ch", Color::rgb(148, 163, 184), None);
            draw_text(bx + 395, list_y + 2, "Security", Color::rgb(148, 163, 184), None);
            draw_text(bx + 485, list_y + 2, "State", Color::rgb(148, 163, 184), None);

            let mut row_y = list_y + 22;
            for (i, net) in self.wifi_networks.iter().enumerate() {
                if row_y + 26 > content_y + 220 {
                    break;
                }

                let is_sel = self.selected_wifi_idx == Some(i);
                if is_sel {
                    fill_rect(bx + 10, row_y, bw - 20, 26, Color::rgb(30, 41, 59));
                    draw_rect_outline(bx + 10, row_y, bw - 20, 26, 1, Color::rgb(6, 182, 212));
                }

                draw_icon(bx + 16, row_y + 5, AppIcon::Network);
                draw_text_clipped(
                    bx + 36,
                    row_y + 5,
                    190,
                    &net.ssid,
                    Color::rgb(255, 255, 255),
                    None,
                );

                // Signal bar
                let meter_w = (net.signal_percent as u32 * 50) / 100;
                fill_rect(bx + 230, row_y + 8, 50, 8, Color::rgb(51, 65, 85));
                fill_rect(bx + 230, row_y + 8, meter_w, 8, Color::rgb(16, 185, 129));
                draw_text(
                    bx + 284,
                    row_y + 5,
                    &format!("{}%", net.signal_percent),
                    Color::rgb(148, 163, 184),
                    None,
                );

                // Band & Channel
                draw_text(
                    bx + 310,
                    row_y + 5,
                    &format!("{} (ch{})", net.band, net.channel),
                    Color::rgb(203, 213, 225),
                    None,
                );

                // Security
                draw_text_clipped(bx + 395, row_y + 5, 85, &net.security, Color::rgb(203, 213, 225), None);

                // State
                let (st_str, st_color) = if net.is_connected {
                    ("Connected", Color::rgb(34, 197, 94))
                } else {
                    ("Available", Color::rgb(148, 163, 184))
                };
                draw_text(bx + 485, row_y + 5, st_str, st_color, None);

                row_y += 28;
            }

            // Connection Box Area
            let conn_y = content_y + 232;
            fill_rect(bx + 10, conn_y, bw - 20, 48, Color::rgb(24, 33, 47));
            draw_rect_outline(bx + 10, conn_y, bw - 20, 48, 1, Color::rgb(51, 65, 85));

            let sel_name = match self.selected_wifi_idx {
                Some(i) if i < self.wifi_networks.len() => &self.wifi_networks[i].ssid[..],
                _ => "(No network selected)",
            };
            draw_text(bx + 16, conn_y + 6, &format!("Selected: {}", sel_name), Color::rgb(241, 245, 249), None);

            draw_text(bx + 16, conn_y + 26, "Password:", Color::rgb(148, 163, 184), None);
            
            // Password input box
            let pwd_box_bg = if self.password_active {
                Color::rgb(15, 23, 42)
            } else {
                Color::rgb(30, 41, 59)
            };
            fill_rect(bx + 90, conn_y + 22, 220, 20, pwd_box_bg);
            let pwd_border = if self.password_active {
                Color::rgb(6, 182, 212)
            } else {
                Color::rgb(71, 85, 105)
            };
            draw_rect_outline(bx + 90, conn_y + 22, 220, 20, 1, pwd_border);

            // Render masked password
            let masked: String = self.password_input.chars().map(|_| '*').collect();
            let cursor = if self.password_active { "_" } else { "" };
            draw_text(
                bx + 95,
                conn_y + 24,
                &format!("{}{}", masked, cursor),
                Color::rgb(255, 255, 255),
                None,
            );

            // Connect button
            let conn_btn_color = if self.is_connecting() {
                Color::rgb(245, 158, 11)
            } else {
                Color::rgb(16, 185, 129)
            };
            let conn_btn_text = if self.is_connecting() { "Connecting..." } else { "Connect" };
            fill_rect(bx + 325, conn_y + 20, 80, 24, conn_btn_color);
            let conn_text_x = if self.is_connecting() { bx + 328 } else { bx + 342 };
            draw_text(conn_text_x, conn_y + 24, conn_btn_text, Color::rgb(255, 255, 255), None);

            // Disconnect button
            fill_rect(bx + 415, conn_y + 20, 95, 24, Color::rgb(239, 68, 68));
            draw_text(bx + 426, conn_y + 24, "Disconnect", Color::rgb(255, 255, 255), None);
        }

        // Status bar at bottom
        let status_y = by + bh as i32 - 22;
        fill_rect(bx, status_y, bw, 22, Color::rgb(30, 41, 59));
        let status = if self.status_msg.is_empty() {
            "Ready. Host Bridge provides real physical Wi-Fi scanning & connection.".to_string()
        } else {
            self.status_msg.clone()
        };
        draw_text_clipped(bx + 8, status_y + 4, bw - 16, &status, Color::rgb(148, 163, 184), None);
    }
}
