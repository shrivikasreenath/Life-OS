use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use spin::Mutex;

#[derive(Debug, Clone)]
pub enum FsNode {
    File {
        name: String,
        content: Vec<u8>,
        size: usize,
    },
    Directory {
        name: String,
        children: BTreeMap<String, FsNode>,
    },
}

impl FsNode {
    pub fn new_file(name: &str, content: &[u8]) -> Self {
        FsNode::File {
            name: name.to_string(),
            content: content.to_vec(),
            size: content.len(),
        }
    }

    pub fn new_dir(name: &str) -> Self {
        FsNode::Directory {
            name: name.to_string(),
            children: BTreeMap::new(),
        }
    }

    pub fn is_dir(&self) -> bool {
        matches!(self, FsNode::Directory { .. })
    }

    pub fn name(&self) -> &str {
        match self {
            FsNode::File { name, .. } => name,
            FsNode::Directory { name, .. } => name,
        }
    }

    pub fn size(&self) -> usize {
        match self {
            FsNode::File { size, .. } => *size,
            FsNode::Directory { children, .. } => children.len(),
        }
    }
}

pub struct VirtualFileSystem {
    root: FsNode,
    current_path: String,
}

impl VirtualFileSystem {
    pub fn new() -> Self {
        let mut root = FsNode::new_dir("");

        // Construct default directory tree
        if let FsNode::Directory { ref mut children, .. } = root {
            // /home
            let mut home_dir = FsNode::new_dir("home");
            if let FsNode::Directory { children: ref mut home_children, .. } = home_dir {
                // /home/shrivika
                let mut shrivika_dir = FsNode::new_dir("shrivika");
                if let FsNode::Directory { children: ref mut shrivika_children, .. } = shrivika_dir {
                    shrivika_children.insert(
                        "README.txt".to_string(),
                        FsNode::new_file(
                            "README.txt",
                            b"Welcome to Life OS!\nAn autonomous bare-metal Rust operating system developed for Shrivika's college project.\nFeatures:\n- Custom x86_64 Rust kernel\n- Intel 82540EM networking\n- Modern desktop environment with window manager\n- 4 playable games\n- Web browser and terminal\n- Windows Wi-Fi host bridge integration\n",
                        ),
                    );

                    let mut docs_dir = FsNode::new_dir("Documents");
                    if let FsNode::Directory { children: ref mut docs_children, .. } = docs_dir {
                        docs_children.insert(
                            "Project_Abstract.txt".to_string(),
                            FsNode::new_file(
                                "Project_Abstract.txt",
                                b"PROJECT: LIFE OS\nAUTHOR: Shrivika\nSUPERVISOR: Department of Computer Science & Engineering\n\nLife OS demonstrates bare-metal systems programming in Rust without standard C runtime, Linux, or Windows dependencies. It achieves sub-10MB bootable ISO footprint with full graphical desktop, preemptive/cooperative multitasking, and genuine TCP/IP connectivity.",
                            ),
                        );
                    }
                    shrivika_children.insert("Documents".to_string(), docs_dir);

                    let mut proj_dir = FsNode::new_dir("Projects");
                    if let FsNode::Directory { children: ref mut proj_children, .. } = proj_dir {
                        proj_children.insert(
                            "Kernel_Architecture.txt".to_string(),
                            FsNode::new_file(
                                "Kernel_Architecture.txt",
                                b"Life OS Kernel Subsystems:\n- Bootloader: Limine x86_64 Protocol\n- Memory: Physical Frame Allocator + 32MB Dynamic Heap\n- Interrupts: 8259 PIC + IDT (Timer, Keyboard, Mouse)\n- Graphics: 32-bit Linear Framebuffer with Double-Buffering\n- Network: e1000 Driver + smoltcp TCP/IP stack\n",
                            ),
                        );
                    }
                    shrivika_children.insert("Projects".to_string(), proj_dir);
                }
                home_children.insert("shrivika".to_string(), shrivika_dir);
            }
            children.insert("home".to_string(), home_dir);

            // /etc
            let mut etc_dir = FsNode::new_dir("etc");
            if let FsNode::Directory { children: ref mut etc_children, .. } = etc_dir {
                etc_children.insert(
                    "os-release".to_string(),
                    FsNode::new_file(
                        "os-release",
                        b"NAME=\"Life OS\"\nVERSION=\"1.0.0\"\nID=life-os\nPRETTY_NAME=\"Life OS 1.0 (x86_64 Bare-Metal Rust)\"\nOWNER=\"Shrivika\"\nUSER=\"shrivika\"\nARCH=\"x86_64\"\n",
                    ),
                );
                etc_children.insert(
                    "hostname".to_string(),
                    FsNode::new_file("hostname", b"life-os\n"),
                );
            }
            children.insert("etc".to_string(), etc_dir);

            // /system
            let mut sys_dir = FsNode::new_dir("system");
            if let FsNode::Directory { children: ref mut sys_children, .. } = sys_dir {
                sys_children.insert(
                    "version.txt".to_string(),
                    FsNode::new_file("version.txt", b"Life OS Kernel v1.0.0-release\nCompiled with rustc nightly (x86_64-unknown-none)\nTarget: Oracle VirtualBox / Bare-Metal x86_64\n"),
                );
            }
            children.insert("system".to_string(), sys_dir);
        }

        Self {
            root,
            current_path: "/home/shrivika".to_string(),
        }
    }

    pub fn get_pwd(&self) -> &str {
        &self.current_path
    }

    pub fn cd(&mut self, path: &str) -> bool {
        let target = self.resolve_path(path);
        if target == "/" {
            self.current_path = "/".to_string();
            return true;
        }

        if let Some(node) = self.find_node(&target) {
            if node.is_dir() {
                self.current_path = target;
                return true;
            }
        }
        false
    }

    pub fn ls(&self, path: Option<&str>) -> Vec<(String, bool, usize)> {
        let target = match path {
            Some(p) => self.resolve_path(p),
            None => self.current_path.clone(),
        };

        let mut items = Vec::new();
        if let Some(FsNode::Directory { children, .. }) = self.find_node(&target) {
            for (name, node) in children {
                items.push((name.clone(), node.is_dir(), node.size()));
            }
        }
        items
    }

    pub fn read_file(&self, path: &str) -> Option<String> {
        let target = self.resolve_path(path);
        if let Some(FsNode::File { content, .. }) = self.find_node(&target) {
            Some(String::from_utf8_lossy(content).into_owned())
        } else {
            None
        }
    }

    pub fn create_file(&mut self, path: &str, content: &[u8]) -> bool {
        let (dir_path, file_name) = self.split_parent(path);
        if let Some(FsNode::Directory { children, .. }) = self.find_node_mut(&dir_path) {
            children.insert(file_name.clone(), FsNode::new_file(&file_name, content));
            true
        } else {
            false
        }
    }

    pub fn create_dir(&mut self, path: &str) -> bool {
        let (dir_path, folder_name) = self.split_parent(path);
        if let Some(FsNode::Directory { children, .. }) = self.find_node_mut(&dir_path) {
            children.insert(folder_name.clone(), FsNode::new_dir(&folder_name));
            true
        } else {
            false
        }
    }

    pub fn delete(&mut self, path: &str) -> bool {
        let (dir_path, name) = self.split_parent(path);
        if let Some(FsNode::Directory { children, .. }) = self.find_node_mut(&dir_path) {
            children.remove(&name).is_some()
        } else {
            false
        }
    }

    pub fn rename(&mut self, old_path: &str, new_name: &str) -> bool {
        let (dir_path, name) = self.split_parent(old_path);
        if let Some(FsNode::Directory { children, .. }) = self.find_node_mut(&dir_path) {
            if let Some(mut node) = children.remove(&name) {
                match node {
                    FsNode::File { ref mut name, .. } => *name = new_name.to_string(),
                    FsNode::Directory { ref mut name, .. } => *name = new_name.to_string(),
                }
                children.insert(new_name.to_string(), node);
                return true;
            }
        }
        false
    }

    fn resolve_path(&self, path: &str) -> String {
        if path.starts_with('/') {
            self.normalize_path(path)
        } else if path == "~" || path.starts_with("~/") {
            let sub = if path.len() > 2 { &path[2..] } else { "" };
            self.normalize_path(&alloc::format!("/home/shrivika/{}", sub))
        } else {
            let combined = alloc::format!("{}/{}", self.current_path, path);
            self.normalize_path(&combined)
        }
    }

    fn normalize_path(&self, path: &str) -> String {
        let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty() && *p != ".").collect();
        let mut resolved: Vec<&str> = Vec::new();

        for part in parts {
            if part == ".." {
                resolved.pop();
            } else {
                resolved.push(part);
            }
        }

        if resolved.is_empty() {
            "/".to_string()
        } else {
            alloc::format!("/{}", resolved.join("/"))
        }
    }

    fn split_parent(&self, path: &str) -> (String, String) {
        let resolved = self.resolve_path(path);
        if let Some(idx) = resolved.rfind('/') {
            let parent = if idx == 0 { "/" } else { &resolved[..idx] };
            let name = &resolved[idx + 1..];
            (parent.to_string(), name.to_string())
        } else {
            ("/".to_string(), resolved)
        }
    }

    fn find_node(&self, path: &str) -> Option<&FsNode> {
        if path == "/" || path.is_empty() {
            return Some(&self.root);
        }

        let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
        let mut cur = &self.root;

        for part in parts {
            match cur {
                FsNode::Directory { children, .. } => {
                    if let Some(node) = children.get(part) {
                        cur = node;
                    } else if let Some((_, node)) = children.iter().find(|(k, _)| k.eq_ignore_ascii_case(part)) {
                        cur = node;
                    } else {
                        return None;
                    }
                }
                FsNode::File { .. } => return None,
            }
        }
        Some(cur)
    }

    fn find_node_mut(&mut self, path: &str) -> Option<&mut FsNode> {
        if path == "/" || path.is_empty() {
            return Some(&mut self.root);
        }

        let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
        let mut cur = &mut self.root;

        for part in parts {
            match cur {
                FsNode::Directory { children, .. } => {
                    cur = children.get_mut(part)?;
                }
                FsNode::File { .. } => return None,
            }
        }
        Some(cur)
    }
}

pub static VFS: Mutex<VirtualFileSystem> = Mutex::new(VirtualFileSystem {
    root: FsNode::Directory {
        name: String::new(),
        children: BTreeMap::new(),
    },
    current_path: String::new(),
});

pub fn init() {
    *VFS.lock() = VirtualFileSystem::new();
    crate::serial_println!("[fs] Virtual File System (LifeFS) initialized at /home/shrivika");
}
