pub mod desktop;
pub mod window;

use crate::drivers::keyboard::pop_key;
use crate::drivers::mouse::get_mouse_state;
use crate::graphics::canvas::draw_cursor;
use crate::graphics::framebuffer::GFX;
use crate::gui::desktop::DESKTOP;
use crate::gui::window::AppId;

pub fn init() {
    let mut desktop = DESKTOP.lock();
    // Launch About window and Terminal by default on first boot!
    desktop.open_app(AppId::About);
    desktop.open_app(AppId::Terminal);
    crate::serial_println!("[gui] Desktop and Window Manager initialized");
}

pub fn update_and_render() {
    // 1. Process keyboard inputs for active window
    while let Some(key) = pop_key() {
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

    // 2. Update background app animations (Pong, Snake)
    crate::apps::update_apps();

    // 3. Process mouse inputs & window interactions
    let mouse = get_mouse_state();
    {
        let mut desktop = DESKTOP.lock();
        desktop.handle_mouse(mouse);

        // A. Render Wallpaper
        desktop.render_wallpaper();

        // B. Render Windows
        for w in &desktop.windows {
            if !w.is_minimized {
                w.render_frame();
                crate::apps::render_app(w);
            }
        }

        // C. Render Taskbar
        desktop.render_taskbar();

        // D. Render Start Menu
        desktop.render_start_menu();
    }

    // 4. Render Mouse Cursor on top
    draw_cursor(mouse.x, mouse.y);

    // 5. Flip double buffer to screen
    if let Some(ref gfx) = *GFX.lock() {
        gfx.present();
    }
}
