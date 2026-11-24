use pic8259::ChainedPics;
use spin::Mutex;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

pub const TIMER_INTERRUPT_ID: u8 = PIC_1_OFFSET;
pub const KEYBOARD_INTERRUPT_ID: u8 = PIC_1_OFFSET + 1;
pub const MOUSE_INTERRUPT_ID: u8 = PIC_2_OFFSET + 4; // IRQ 12

pub fn init() {
    unsafe {
        let mut pics = PICS.lock();
        pics.initialize();
        // Explicitly unmask PIC1 and PIC2 interrupts (timer IRQ0, keyboard IRQ1, cascade IRQ2, mouse IRQ12, etc.)
        // This is required because modern UEFI firmware boots with PIC masks set to 0xFF (all disabled)
        pics.write_masks(0x00, 0x00);
    }
}

pub fn notify_end_of_interrupt(interrupt_id: u8) {
    unsafe {
        PICS.lock().notify_end_of_interrupt(interrupt_id);
    }
}
