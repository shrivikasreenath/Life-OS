use limine::memory_map::EntryType;
use limine::request::MemoryMapRequest;
use spin::Mutex;

#[used]
pub static MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

pub const PAGE_SIZE: u64 = 4096;

pub struct FrameAllocator {
    next_frame: u64,
    current_entry_idx: usize,
    total_bytes: u64,
    usable_bytes: u64,
    allocated_bytes: u64,
}

impl FrameAllocator {
    pub const fn new() -> Self {
        Self {
            next_frame: 0,
            current_entry_idx: 0,
            total_bytes: 0,
            usable_bytes: 0,
            allocated_bytes: 0,
        }
    }

    pub fn init(&mut self) {
        let response = match MEMORY_MAP_REQUEST.get_response() {
            Some(resp) => resp,
            None => panic!("Limine MemoryMap request failed"),
        };

        let entries = response.entries();
        for entry in entries {
            self.total_bytes += entry.length;
            if entry.entry_type == EntryType::USABLE {
                self.usable_bytes += entry.length;
            }
        }

        // Find the first usable entry
        for (i, entry) in entries.iter().enumerate() {
            if entry.entry_type == EntryType::USABLE && entry.length >= PAGE_SIZE {
                self.current_entry_idx = i;
                self.next_frame = (entry.base + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
                break;
            }
        }

        crate::serial_println!(
            "[mm] Frame Allocator: Total RAM: {} MB, Usable: {} MB",
            self.total_bytes / (1024 * 1024),
            self.usable_bytes / (1024 * 1024)
        );
    }

    pub fn allocate_frame(&mut self) -> Option<u64> {
        self.allocate_contiguous_frames(1)
    }

    pub fn allocate_contiguous_frames(&mut self, count: usize) -> Option<u64> {
        let response = MEMORY_MAP_REQUEST.get_response()?;
        let entries = response.entries();
        let needed_bytes = (count as u64) * PAGE_SIZE;

        while self.current_entry_idx < entries.len() {
            let entry = entries[self.current_entry_idx];
            if entry.entry_type == EntryType::USABLE {
                let entry_end = entry.base + entry.length;
                if self.next_frame + needed_bytes <= entry_end {
                    let phys_addr = self.next_frame;
                    self.next_frame += needed_bytes;
                    self.allocated_bytes += needed_bytes;
                    return Some(phys_addr);
                }
            }

            // Advance to next entry
            self.current_entry_idx += 1;
            if self.current_entry_idx < entries.len() {
                let next_entry = entries[self.current_entry_idx];
                self.next_frame = (next_entry.base + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
            }
        }

        None
    }

    pub fn total_bytes(&self) -> u64 {
        self.total_bytes
    }

    pub fn usable_bytes(&self) -> u64 {
        self.usable_bytes
    }

    pub fn allocated_bytes(&self) -> u64 {
        self.allocated_bytes
    }

    pub fn free_bytes(&self) -> u64 {
        if self.usable_bytes > self.allocated_bytes {
            self.usable_bytes - self.allocated_bytes
        } else {
            0
        }
    }
}

pub static FRAME_ALLOCATOR: Mutex<FrameAllocator> = Mutex::new(FrameAllocator::new());

pub fn init() {
    FRAME_ALLOCATOR.lock().init();
}

pub fn alloc_frame() -> Option<u64> {
    FRAME_ALLOCATOR.lock().allocate_frame()
}

pub fn alloc_contiguous_frames(count: usize) -> Option<u64> {
    FRAME_ALLOCATOR.lock().allocate_contiguous_frames(count)
}
