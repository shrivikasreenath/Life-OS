pub mod desktop;
pub mod window;

use crate::drivers::keyboard::pop_key;
use crate::drivers::mouse::get_mouse_state;
use crate::graphics::canvas::draw_cursor;
use crate::graphics::framebuffer::GFX;
use crate::gui::desktop::DESKTOP;
use crate::gui::window::AppId;
use pc_keyboard::{DecodedKey, KeyCode};

pub fn init() {
    let mut desktop = DESKTOP.lock();
    // Launch About window and Terminal by default on first boot!
    desktop.open_app(AppId::About);
    desktop.open_app(AppId::Terminal);
    crate::serial_println!("[gui] Desktop and Window Manager initialized");
}

pub fn update_and_render() {
    // 1. Process keyboard inputs for active window or global system shortcuts
    while let Some(key) = pop_key() {
        let mut handled = false;
        match key {
            DecodedKey::RawKey(KeyCode::F1) => {
                let mut desktop = DESKTOP.lock();
                desktop.start_menu_open = !desktop.start_menu_open;
                desktop.context_menu_open = false;
                desktop.calendar_open = false;
                handled = true;
            }
            DecodedKey::RawKey(KeyCode::F2) => {
                let mut desktop = DESKTOP.lock();
                desktop.calendar_open = !desktop.calendar_open;
                desktop.start_menu_open = false;
                desktop.context_menu_open = false;
                handled = true;
            }
            DecodedKey::RawKey(KeyCode::F3) => {
                let mut desktop = DESKTOP.lock();
                desktop.context_menu_open = !desktop.context_menu_open;
                desktop.context_menu_x = 220;
                desktop.context_menu_y = 180;
                desktop.start_menu_open = false;
                desktop.calendar_open = false;
                handled = true;
            }
            DecodedKey::RawKey(KeyCode::F4) => {
                let mut desktop = DESKTOP.lock();
                if let Some(id) = desktop.active_window_id {
                    desktop.close_window(id);
                    handled = true;
                }
            }
            DecodedKey::RawKey(KeyCode::F5) => {
                let mut desktop = DESKTOP.lock();
                desktop.cycle_wallpaper();
                handled = true;
            }
            DecodedKey::RawKey(KeyCode::F6) => {
                let mut desktop = DESKTOP.lock();
                if !desktop.windows.is_empty() {
                    let cur_id = desktop.active_window_id.unwrap_or(0);
                    let idx = desktop.windows.iter().position(|w| w.id == cur_id).unwrap_or(0);
                    let next_idx = (idx + 1) % desktop.windows.len();
                    let next_id = desktop.windows[next_idx].id;
                    desktop.focus_window(next_id);
                    handled = true;
                }
            }
            DecodedKey::RawKey(KeyCode::F7) => {
                let mut desktop = DESKTOP.lock();
                desktop.open_app(AppId::Games);
                handled = true;
            }
            DecodedKey::RawKey(KeyCode::F8) => {
                let mut desktop = DESKTOP.lock();
                desktop.open_app(AppId::Browser);
                handled = true;
            }
            DecodedKey::RawKey(KeyCode::F9) => {
                let mut desktop = DESKTOP.lock();
                desktop.open_app(AppId::SysInfo);
                handled = true;
            }
            DecodedKey::RawKey(KeyCode::F10) => {
                let mut desktop = DESKTOP.lock();
                desktop.open_app(AppId::Settings);
                handled = true;
            }
            DecodedKey::RawKey(KeyCode::F12) => {
                let mut desktop = DESKTOP.lock();
                if let Some(id) = desktop.active_window_id {
                    desktop.toggle_maximize(id);
                    handled = true;
                }
            }
            DecodedKey::RawKey(KeyCode::Escape) | DecodedKey::Unicode('\x1b') => {
                let mut desktop = DESKTOP.lock();
                if desktop.start_menu_open || desktop.context_menu_open || desktop.calendar_open {
                    desktop.start_menu_open = false;
                    desktop.context_menu_open = false;
                    desktop.calendar_open = false;
                    handled = true;
                }
            }
            _ => {}
        }

        // Start Menu number hotkeys (1..8)
        if !handled {
            let mut start_menu_handled = false;
            {
                let mut desktop = DESKTOP.lock();
                if desktop.start_menu_open {
                    match key {
                        DecodedKey::Unicode('1') => { desktop.open_app(AppId::Terminal); desktop.start_menu_open = false; start_menu_handled = true; }
                        DecodedKey::Unicode('2') => { desktop.open_app(AppId::Browser); desktop.start_menu_open = false; start_menu_handled = true; }
                        DecodedKey::Unicode('3') => { desktop.open_app(AppId::FileManager); desktop.start_menu_open = false; start_menu_handled = true; }
                        DecodedKey::Unicode('4') => { desktop.open_app(AppId::NetworkManager); desktop.start_menu_open = false; start_menu_handled = true; }
                        DecodedKey::Unicode('5') => { desktop.open_app(AppId::Games); desktop.start_menu_open = false; start_menu_handled = true; }
                        DecodedKey::Unicode('6') => { desktop.open_app(AppId::SysInfo); desktop.start_menu_open = false; start_menu_handled = true; }
                        DecodedKey::Unicode('7') => { desktop.open_app(AppId::Settings); desktop.start_menu_open = false; start_menu_handled = true; }
                        DecodedKey::Unicode('8') => { desktop.open_app(AppId::About); desktop.start_menu_open = false; start_menu_handled = true; }
                        _ => {}
                    }
                }
            }
            if start_menu_handled {
                handled = true;
            }
        }

        if !handled {
            let active_app = {
                let desktop = DESKTOP.lock();
                desktop.active_window_id.and_then(|id| {
                    desktop.windows.iter().find(|win| win.id == id).map(|w| w.app_id)
                })
            };
            if let Some(app_id) = active_app {
                crate::apps::dispatch_key(app_id, key);
            }
        }
    }

    // 2. Update background app animations (Pong, Snake)
    crate::apps::update_apps();

    // 3. Process mouse inputs & window interactions
    let mouse = get_mouse_state();
    let pending_click = {
        let mut desktop = DESKTOP.lock();
        desktop.handle_mouse(mouse)
    };
    if let Some((app_id, local_x, local_y, bw, bh)) = pending_click {
        crate::apps::dispatch_click(app_id, local_x, local_y, bw, bh);
    }
    {
        let mut desktop = DESKTOP.lock();

        // A. Render Wallpaper
        desktop.render_wallpaper();

        // B. Render Windows
        for w in &desktop.windows {
            if !w.is_minimized {
                w.render_frame();
                crate::apps::render_app(w);
            }
        }

        // C. Render Context Menu (above windows)
        if desktop.context_menu_open {
            desktop.render_context_menu();
        }

        // D. Render Taskbar
        desktop.render_taskbar();

        // E. Render Start Menu
        desktop.render_start_menu();
    }

    // 4. Render Mouse Cursor on top
    draw_cursor(mouse.x, mouse.y);

    // 5. Flip double buffer to screen
    if let Some(ref gfx) = *GFX.lock() {
        gfx.present();
    }
}
