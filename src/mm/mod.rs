pub mod frame;
pub mod heap;
pub mod paging;

pub fn init() {
    paging::init();
    frame::init();
    heap::init();
    crate::serial_println!("[mm] Memory management subsystems fully initialized");
}
