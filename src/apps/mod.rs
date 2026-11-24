pub mod about;
pub mod browser;
pub mod file_manager;
pub mod games;
pub mod network_manager;
pub mod settings;
pub mod sysinfo;
pub mod terminal;

use crate::gui::window::{AppId, Window};
use pc_keyboard::DecodedKey;

pub fn init() {
    terminal::TERMINAL.lock().init();
    browser::BROWSER.lock().init();
    games::init();
}

pub fn render_app(win: &Window) {
    match win.app_id {
        AppId::Terminal => terminal::TERMINAL.lock().render(win),
        AppId::Browser => browser::BROWSER.lock().render(win),
        AppId::FileManager => file_manager::FILE_MANAGER.lock().render(win),
        AppId::NetworkManager => network_manager::NETWORK_MANAGER.lock().render(win),
        AppId::Games => {
            if let Some(ref suite) = *games::GAMES_SUITE.lock() {
                suite.render(win);
            }
        }
        AppId::SysInfo => sysinfo::SYSINFO.lock().render(win),
        AppId::Settings => settings::SETTINGS.lock().render(win),
        AppId::About => about::ABOUT.lock().render(win),
    }
}

pub fn dispatch_key(app_id: AppId, key: DecodedKey) {
    match app_id {
        AppId::Terminal => terminal::TERMINAL.lock().handle_key(key),
        AppId::Browser => browser::BROWSER.lock().handle_key(key),
        AppId::FileManager => file_manager::FILE_MANAGER.lock().handle_key(key),
        AppId::Games => {
            if let Some(ref mut suite) = *games::GAMES_SUITE.lock() {
                suite.handle_key(key);
            }
        }
        AppId::NetworkManager => network_manager::NETWORK_MANAGER.lock().handle_key(key),
        _ => {}
    }
}

pub fn dispatch_click(app_id: AppId, local_x: i32, local_y: i32, win: &Window) {
    match app_id {
        AppId::FileManager => file_manager::FILE_MANAGER.lock().handle_click(local_x, local_y, win),
        AppId::NetworkManager => network_manager::NETWORK_MANAGER.lock().handle_click(local_x, local_y),
        AppId::Games => {
            if let Some(ref mut suite) = *games::GAMES_SUITE.lock() {
                suite.handle_click(local_x, local_y);
            }
        }
        AppId::Settings => settings::SETTINGS.lock().handle_click(local_x, local_y),
        _ => {}
    }
}

pub fn update_apps() {
    if let Some(ref mut suite) = *games::GAMES_SUITE.lock() {
        suite.update();
    }
}
