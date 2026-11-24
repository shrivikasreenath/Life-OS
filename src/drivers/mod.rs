pub mod e1000;
pub mod keyboard;
pub mod mouse;
pub mod pci;
pub mod rtc;

pub fn init() {
    keyboard::init();
    mouse::init();
    let net_found = e1000::init();
    if net_found {
        crate::serial_println!("[drivers] Intel 82540EM Gigabit NIC ready");
    } else {
        crate::serial_println!("[drivers] NIC not detected or running in non-networking mode");
    }
}
