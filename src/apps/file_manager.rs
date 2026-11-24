use crate::fs::vfs::VFS;
use crate::graphics::canvas::{draw_rect_outline, draw_text, draw_text_clipped, fill_rect};
use crate::graphics::framebuffer::Color;
use crate::graphics::icons::{draw_icon, AppIcon};
use crate::gui::window::Window;
use alloc::format;
use alloc::string::{String, ToString};
use spin::Mutex;

pub struct FileManagerApp {
    selected_index: Option<usize>,
    viewing_file: Option<(String, String)>, // (filename, content)
    status_msg: String,
}

pub static FILE_MANAGER: Mutex<FileManagerApp> = Mutex::new(FileManagerApp::new());

impl FileManagerApp {
    pub const fn new() -> Self {
        Self {
            selected_index: None,
            viewing_file: None,
            status_msg: String::new(),
        }
    }

    pub fn handle_click(&mut self, local_x: i32, local_y: i32, bw: u32, _bh: u32) {

        // 1. Toolbar clicks (Top 32px)
        if local_y < 32 {
            if local_x >= 6 && local_x <= 70 {
                // "New File"
                let count = VFS.lock().ls(None).len();
                let fname = format!("note_{}.txt", count + 1);
                VFS.lock().create_file(&fname, b"Created inside Life OS File Manager.\n");
                self.status_msg = format!("Created {}", fname);
            } else if local_x >= 76 && local_x <= 150 {
                // "New Folder"
                let count = VFS.lock().ls(None).len();
                let dname = format!("folder_{}", count + 1);
                VFS.lock().create_dir(&dname);
                self.status_msg = format!("Created folder {}", dname);
            } else if local_x >= 156 && local_x <= 210 {
                // "Delete"
                if let Some(idx) = self.selected_index {
                    let items = VFS.lock().ls(None);
                    if idx < items.len() {
                        let name = &items[idx].0;
                        VFS.lock().delete(name);
                        self.status_msg = format!("Deleted {}", name);
                        self.selected_index = None;
                    }
                }
            } else if local_x >= 216 && local_x <= 270 {
                // "Up .."
                VFS.lock().cd("..");
                self.selected_index = None;
                self.viewing_file = None;
            }
            return;
        }

        // 2. File viewer close button click
        if self.viewing_file.is_some() {
            let close_btn_x = bw as i32 - 30;
            if local_x >= close_btn_x && local_y >= 36 && local_y <= 56 {
                self.viewing_file = None;
            }
            return;
        }

        // 3. File list item clicks
        let list_y = local_y - 36;
        let item_h = 24;
        if list_y >= 0 {
            let idx = (list_y / item_h) as usize;
            let items = VFS.lock().ls(None);
            if idx < items.len() {
                if self.selected_index == Some(idx) {
                    // Double click / Open
                    let (name, is_dir, _) = &items[idx];
                    if *is_dir {
                        VFS.lock().cd(name);
                        self.selected_index = None;
                    } else {
                        // Open text file
                        if let Some(content) = VFS.lock().read_file(name) {
                            self.viewing_file = Some((name.clone(), content));
                        }
                    }
                } else {
                    self.selected_index = Some(idx);
                    self.status_msg = format!("Selected: {}", items[idx].0);
                }
            }
        }
    }

    pub fn handle_key(&mut self, key: pc_keyboard::DecodedKey) {
        use pc_keyboard::KeyCode;
        if self.viewing_file.is_some() {
            match key {
                pc_keyboard::DecodedKey::RawKey(KeyCode::Escape)
                | pc_keyboard::DecodedKey::Unicode('\x1b')
                | pc_keyboard::DecodedKey::Unicode('q') => {
                    self.viewing_file = None;
                }
                _ => {}
            }
            return;
        }

        match key {
            pc_keyboard::DecodedKey::RawKey(KeyCode::ArrowDown) | pc_keyboard::DecodedKey::Unicode('j') => {
                let items = VFS.lock().ls(None);
                if !items.is_empty() {
                    let next = match self.selected_index {
                        Some(i) if i + 1 < items.len() => i + 1,
                        Some(_) => 0,
                        None => 0,
                    };
                    self.selected_index = Some(next);
                    self.status_msg = format!("Selected: {}", items[next].0);
                }
            }
            pc_keyboard::DecodedKey::RawKey(KeyCode::ArrowUp) | pc_keyboard::DecodedKey::Unicode('k') => {
                let items = VFS.lock().ls(None);
                if !items.is_empty() {
                    let prev = match self.selected_index {
                        Some(i) if i > 0 => i - 1,
                        Some(_) => items.len().saturating_sub(1),
                        None => 0,
                    };
                    self.selected_index = Some(prev);
                    self.status_msg = format!("Selected: {}", items[prev].0);
                }
            }
            pc_keyboard::DecodedKey::Unicode('\n') | pc_keyboard::DecodedKey::Unicode('\r') => {
                if let Some(idx) = self.selected_index {
                    let items = VFS.lock().ls(None);
                    if idx < items.len() {
                        let (name, is_dir, _) = &items[idx];
                        if *is_dir {
                            VFS.lock().cd(name);
                            self.selected_index = None;
                        } else {
                            if let Some(content) = VFS.lock().read_file(name) {
                                self.viewing_file = Some((name.clone(), content));
                            }
                        }
                    }
                }
            }
            pc_keyboard::DecodedKey::Unicode('u') | pc_keyboard::DecodedKey::RawKey(KeyCode::Backspace) => {
                VFS.lock().cd("..");
                self.selected_index = None;
                self.viewing_file = None;
            }
            pc_keyboard::DecodedKey::Unicode('n') | pc_keyboard::DecodedKey::Unicode('f') => {
                let count = VFS.lock().ls(None).len();
                let fname = format!("note_{}.txt", count + 1);
                VFS.lock().create_file(&fname, b"Created inside Life OS File Manager.\n");
                self.status_msg = format!("Created {}", fname);
            }
            pc_keyboard::DecodedKey::Unicode('d') | pc_keyboard::DecodedKey::Unicode('m') => {
                let count = VFS.lock().ls(None).len();
                let dname = format!("folder_{}", count + 1);
                VFS.lock().create_dir(&dname);
                self.status_msg = format!("Created folder {}", dname);
            }
            pc_keyboard::DecodedKey::Unicode('x') | pc_keyboard::DecodedKey::RawKey(KeyCode::Delete) => {
                if let Some(idx) = self.selected_index {
                    let items = VFS.lock().ls(None);
                    if idx < items.len() {
                        let name = &items[idx].0;
                        VFS.lock().delete(name);
                        self.status_msg = format!("Deleted {}", name);
                        self.selected_index = None;
                    }
                }
            }
            _ => {}
        }
    }

    pub fn render(&self, win: &Window) {
        let bx = win.body_x();
        let by = win.body_y();
        let bw = win.body_width();
        let bh = win.body_height();

        // 1. Toolbar
        fill_rect(bx, by, bw, 32, Color::rgb(30, 41, 59));

        // Buttons
        // New File
        fill_rect(bx + 6, by + 5, 64, 22, Color::rgb(6, 182, 212));
        draw_text(bx + 12, by + 8, "+ File", Color::rgb(255, 255, 255), None);

        // New Folder
        fill_rect(bx + 76, by + 5, 74, 22, Color::rgb(16, 185, 129));
        draw_text(bx + 82, by + 8, "+ Folder", Color::rgb(255, 255, 255), None);

        // Delete
        fill_rect(bx + 156, by + 5, 54, 22, Color::rgb(239, 68, 68));
        draw_text(bx + 164, by + 8, "Delete", Color::rgb(255, 255, 255), None);

        // Up ..
        fill_rect(bx + 216, by + 5, 54, 22, Color::rgb(71, 85, 105));
        draw_text(bx + 228, by + 8, "Up ..", Color::rgb(255, 255, 255), None);

        // Current Path display
        let pwd = VFS.lock().get_pwd().to_string();
        let path_x = bx + 280;
        let path_w = if bw > 290 { bw - 290 } else { 50 };
        fill_rect(path_x, by + 5, path_w, 22, Color::rgb(15, 23, 42));
        draw_text_clipped(path_x + 6, by + 8, path_w - 12, &pwd, Color::rgb(226, 232, 240), None);

        // 2. Main Area (File list or File viewer)
        let content_y = by + 34;
        let content_h = if bh > 54 { bh - 54 } else { 0 };

        if let Some((ref fname, ref fcontent)) = self.viewing_file {
            // Text Viewer overlay
            fill_rect(bx, content_y, bw, content_h, Color::rgb(15, 23, 42));

            // Header
            fill_rect(bx, content_y, bw, 24, Color::rgb(30, 41, 59));
            draw_text(bx + 8, content_y + 4, &format!("Viewing: {}", fname), Color::rgb(6, 182, 212), None);

            // Close button
            fill_rect(bx + bw as i32 - 24, content_y + 2, 20, 20, Color::rgb(239, 68, 68));
            draw_text(bx + bw as i32 - 18, content_y + 4, "X", Color::rgb(255, 255, 255), None);

            // Content
            let mut text_y = content_y + 30;
            for line in fcontent.lines() {
                if text_y + 16 > by + bh as i32 - 20 {
                    break;
                }
                draw_text_clipped(bx + 12, text_y, bw - 24, line, Color::rgb(241, 245, 249), None);
                text_y += 18;
            }
        } else {
            // File List
            fill_rect(bx, content_y, bw, content_h, Color::rgb(15, 23, 42));

            // Table header
            fill_rect(bx, content_y, bw, 20, Color::rgb(24, 33, 47));
            draw_text(bx + 12, content_y + 2, "Name", Color::rgb(148, 163, 184), None);
            draw_text(bx + 260, content_y + 2, "Type", Color::rgb(148, 163, 184), None);
            draw_text(bx + 340, content_y + 2, "Size", Color::rgb(148, 163, 184), None);

            let items = VFS.lock().ls(None);
            let item_h = 24;
            let mut cur_y = content_y + 22;

            for (i, (name, is_dir, size)) in items.iter().enumerate() {
                if cur_y + item_h > by + bh as i32 - 20 {
                    break;
                }

                let is_selected = self.selected_index == Some(i);
                if is_selected {
                    fill_rect(bx + 2, cur_y, bw - 4, item_h as u32, Color::rgb(30, 41, 59));
                    draw_rect_outline(bx + 2, cur_y, bw - 4, item_h as u32, 1, Color::rgb(6, 182, 212));
                }

                // Icon
                if *is_dir {
                    draw_icon(bx + 8, cur_y + 4, AppIcon::FileManager);
                } else {
                    fill_rect(bx + 10, cur_y + 4, 12, 16, Color::rgb(148, 163, 184)); // Text document icon
                }

                // Name
                let name_color = if *is_dir {
                    Color::rgb(245, 158, 11) // Amber for directories
                } else {
                    Color::rgb(241, 245, 249) // White for files
                };
                draw_text_clipped(bx + 30, cur_y + 4, 220, name, name_color, None);

                // Type
                let type_str = if *is_dir { "Directory" } else { "Text Document" };
                draw_text(bx + 260, cur_y + 4, type_str, Color::rgb(148, 163, 184), None);

                // Size
                let size_str = if *is_dir {
                    format!("{} items", size)
                } else {
                    format!("{} B", size)
                };
                draw_text(bx + 340, cur_y + 4, &size_str, Color::rgb(148, 163, 184), None);

                cur_y += item_h as i32;
            }
        }

        // 3. Status Bar at bottom
        let status_y = by + bh as i32 - 20;
        fill_rect(bx, status_y, bw, 20, Color::rgb(30, 41, 59));
        let items_count = VFS.lock().ls(None).len();
        let status = if self.status_msg.is_empty() {
            format!("{} items in directory | Click to select, double-click to open", items_count)
        } else {
            self.status_msg.clone()
        };
        draw_text_clipped(bx + 8, status_y + 2, bw - 16, &status, Color::rgb(148, 163, 184), None);
    }
}
