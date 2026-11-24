use crate::mm::frame::{alloc_contiguous_frames, PAGE_SIZE};
use crate::mm::paging::phys_to_virt;
use linked_list_allocator::LockedHeap;

pub const HEAP_SIZE: usize = 32 * 1024 * 1024; // 32 Megabytes

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init() {
    let frame_count = (HEAP_SIZE as u64 + PAGE_SIZE - 1) / PAGE_SIZE;
    let phys_start = alloc_contiguous_frames(frame_count as usize)
        .expect("Failed to allocate physical memory for kernel heap");

    let virt_start = phys_to_virt(phys_start) as *mut u8;

    unsafe {
        ALLOCATOR.lock().init(virt_start, HEAP_SIZE);
    }

    crate::serial_println!(
        "[mm] Heap initialized: {} MB at virt: 0x{:016x} (phys: 0x{:016x})",
        HEAP_SIZE / (1024 * 1024),
        virt_start as u64,
        phys_start
    );
}

pub fn heap_total_bytes() -> usize {
    HEAP_SIZE
}

pub fn heap_used_bytes() -> usize {
    // Estimator based on allocator lock
    ALLOCATOR.lock().size()
}

#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("Kernel Allocation Error: {:?}", layout);
}
