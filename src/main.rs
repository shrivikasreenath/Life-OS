#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

extern crate alloc;

use core::panic::PanicInfo;
use limine::BaseRevision;

pub mod apps;
pub mod arch;
pub mod drivers;
pub mod fs;
pub mod graphics;
pub mod gui;
pub mod mm;
pub mod net;
pub mod power;

pub static BASE_REVISION: BaseRevision = BaseRevision::new();

#[no_mangle]
pub extern "C" fn _start() -> ! {
    arch::init();
    mm::init();
    drivers::init();
    fs::init();
    net::init();
    graphics::init();
    apps::init();
    gui::init();

    serial_println!("====================================================");
    serial_println!("            LIFE OS (Bare-Metal Rust Kernel)        ");
    serial_println!("       Owner: Shrivika | Architecture: x86_64       ");
    serial_println!("====================================================");
    serial_println!("[kernel] Desktop environment and main loop active");

    loop {
        net::poll();
        gui::update_and_render();
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("KERNEL PANIC: {}", info);
    loop {
        x86_64::instructions::hlt();
    }
}
