pub mod smoltcp_adapter;
pub mod stack;

pub fn init() {
    stack::init();
}

pub fn poll() {
    stack::poll();
}
