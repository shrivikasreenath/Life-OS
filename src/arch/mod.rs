pub mod gdt;
pub mod idt;
pub mod pic;
pub mod pit;
pub mod serial;

pub fn init() {
    serial::init();
    crate::serial_println!("[arch] Serial port initialized (COM1 115200 8N1)");

    gdt::init();
    crate::serial_println!("[arch] GDT and TSS initialized with IST");

    idt::init();
    crate::serial_println!("[arch] IDT initialized");

    pic::init();
    crate::serial_println!("[arch] 8259 PIC initialized and IRQs remapped to 32..47");

    pit::init();
    crate::serial_println!("[arch] PIT timer initialized at 100 Hz");

    x86_64::instructions::interrupts::enable();
    crate::serial_println!("[arch] Hardware interrupts enabled (IF flag set)");
}
