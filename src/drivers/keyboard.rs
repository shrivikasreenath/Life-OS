use pc_keyboard::{
    layouts, DecodedKey, EventDecoder, HandleControl, ScancodeSet,
    ScancodeSet1,
};
use spin::Mutex;
use x86_64::instructions::port::Port;

const KEY_BUFFER_SIZE: usize = 128;

pub struct KeyQueue {
    buffer: [Option<DecodedKey>; KEY_BUFFER_SIZE],
    read_pos: usize,
    write_pos: usize,
}

impl KeyQueue {
    pub const fn new() -> Self {
        Self {
            buffer: [None; KEY_BUFFER_SIZE],
            read_pos: 0,
            write_pos: 0,
        }
    }

    pub fn push(&mut self, key: DecodedKey) {
        let next = (self.write_pos + 1) % KEY_BUFFER_SIZE;
        if next != self.read_pos {
            self.buffer[self.write_pos] = Some(key);
            self.write_pos = next;
        }
    }

    pub fn pop(&mut self) -> Option<DecodedKey> {
        if self.read_pos == self.write_pos {
            None
        } else {
            let key = self.buffer[self.read_pos].take();
            self.read_pos = (self.read_pos + 1) % KEY_BUFFER_SIZE;
            key
        }
    }
}

pub static KEY_QUEUE: Mutex<KeyQueue> = Mutex::new(KeyQueue::new());

pub struct KeyboardDriver {
    scancode_set: ScancodeSet1,
    decoder: EventDecoder<layouts::Us104Key>,
}

static KEYBOARD: Mutex<Option<KeyboardDriver>> = Mutex::new(None);

pub fn init() {
    // Drain any stale power-on self-test or ACK bytes from PS/2 controller data port
    unsafe {
        let mut status_port = Port::<u8>::new(0x64);
        let mut data_port = Port::<u8>::new(0x60);
        while (status_port.read() & 0x01) != 0 {
            let _ = data_port.read();
        }
    }

    *KEYBOARD.lock() = Some(KeyboardDriver {
        scancode_set: ScancodeSet1::new(),
        decoder: EventDecoder::new(layouts::Us104Key, HandleControl::Ignore),
    });
    crate::serial_println!("[drivers] PS/2 Keyboard initialized (US 104-key)");
}

pub fn on_interrupt() {
    let mut port = Port::<u8>::new(0x60);
    let scancode = unsafe { port.read() };

    if let Some(ref mut kbd) = *KEYBOARD.lock() {
        if let Ok(Some(key_event)) = kbd.scancode_set.advance_state(scancode) {
            if let Some(key) = kbd.decoder.process_keyevent(key_event) {
                KEY_QUEUE.lock().push(key);
            }
        }
    }
}

pub fn pop_key() -> Option<DecodedKey> {
    KEY_QUEUE.lock().pop()
}

pub fn pop_char() -> Option<char> {
    match pop_key() {
        Some(DecodedKey::Unicode(c)) => Some(c),
        _ => None,
    }
}
