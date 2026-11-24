use crate::arch::gdt::DOUBLE_FAULT_IST_INDEX;
use crate::arch::pic::{
    notify_end_of_interrupt, KEYBOARD_INTERRUPT_ID, MOUSE_INTERRUPT_ID, TIMER_INTERRUPT_ID,
};
use crate::arch::pit;
use crate::serial_println;
use x86_64::registers::control::Cr2;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

pub fn init() {
    unsafe {
        let idt: &'static mut InterruptDescriptorTable = &mut *(&raw mut IDT);
        idt.divide_error.set_handler_fn(divide_error_handler);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        idt.general_protection_fault.set_handler_fn(general_protection_fault_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);

        let double_fault_entry = idt.double_fault.set_handler_fn(double_fault_handler);
        double_fault_entry.set_stack_index(DOUBLE_FAULT_IST_INDEX);

        idt[TIMER_INTERRUPT_ID as usize].set_handler_fn(timer_interrupt_handler);
        idt[KEYBOARD_INTERRUPT_ID as usize].set_handler_fn(keyboard_interrupt_handler);
        idt[MOUSE_INTERRUPT_ID as usize].set_handler_fn(mouse_interrupt_handler);

        idt.load();
    }
}

extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    serial_println!("EXCEPTION: DIVIDE BY ZERO\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    serial_println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    serial_println!("EXCEPTION: INVALID OPCODE\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    serial_println!(
        "EXCEPTION: GENERAL PROTECTION FAULT (code: {})\n{:#?}",
        error_code,
        stack_frame
    );
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    let fault_addr = Cr2::read();
    serial_println!(
        "EXCEPTION: PAGE FAULT at {:?}\nError Code: {:?}\n{:#?}",
        fault_addr,
        error_code,
        stack_frame
    );
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    serial_println!(
        "FATAL: DOUBLE FAULT (code: {})\n{:#?}",
        error_code,
        stack_frame
    );
    loop {
        x86_64::instructions::hlt();
    }
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    pit::on_tick();
    notify_end_of_interrupt(TIMER_INTERRUPT_ID);
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    crate::drivers::keyboard::on_interrupt();
    notify_end_of_interrupt(KEYBOARD_INTERRUPT_ID);
}

extern "x86-interrupt" fn mouse_interrupt_handler(_stack_frame: InterruptStackFrame) {
    crate::drivers::mouse::on_interrupt();
    notify_end_of_interrupt(MOUSE_INTERRUPT_ID);
}
