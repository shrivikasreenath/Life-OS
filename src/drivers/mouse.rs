use spin::Mutex;
use x86_64::instructions::port::Port;

#[derive(Debug, Clone, Copy)]
pub struct MouseState {
    pub x: i32,
    pub y: i32,
    pub left_button: bool,
    pub right_button: bool,
    pub middle_button: bool,
}

pub static MOUSE: Mutex<MouseDriver> = Mutex::new(MouseDriver::new());

pub struct MouseDriver {
    packet: [u8; 3],
    packet_index: usize,
    state: MouseState,
    max_x: i32,
    max_y: i32,
}

impl MouseDriver {
    pub const fn new() -> Self {
        Self {
            packet: [0; 3],
            packet_index: 0,
            state: MouseState {
                x: 512,
                y: 384,
                left_button: false,
                right_button: false,
                middle_button: false,
            },
            max_x: 1024,
            max_y: 768,
        }
    }

    pub fn set_bounds(&mut self, width: u32, height: u32) {
        self.max_x = width as i32;
        self.max_y = height as i32;
        self.state.x = self.max_x / 2;
        self.state.y = self.max_y / 2;
    }

    pub fn get_state(&self) -> MouseState {
        self.state
    }

    pub fn handle_byte(&mut self, byte: u8) {
        // First byte must have bit 3 set to 1 for sync
        if self.packet_index == 0 && (byte & 0x08) == 0 {
            return;
        }

        self.packet[self.packet_index] = byte;
        self.packet_index += 1;

        if self.packet_index == 3 {
            self.packet_index = 0;
            self.process_packet();
        }
    }

    fn process_packet(&mut self) {
        let flags = self.packet[0];
        let mut dx = self.packet[1] as i32;
        let mut dy = self.packet[2] as i32;

        if (flags & 0x10) != 0 {
            dx -= 256;
        }
        if (flags & 0x20) != 0 {
            dy -= 256;
        }

        // PS/2 mouse Y is inverted (positive dy means upwards)
        self.state.x += dx;
        self.state.y -= dy;

        // Clamp to screen bounds
        if self.state.x < 0 {
            self.state.x = 0;
        }
        if self.state.x >= self.max_x {
            self.state.x = self.max_x - 1;
        }
        if self.state.y < 0 {
            self.state.y = 0;
        }
        if self.state.y >= self.max_y {
            self.state.y = self.max_y - 1;
        }

        self.state.left_button = (flags & 0x01) != 0;
        self.state.right_button = (flags & 0x02) != 0;
        self.state.middle_button = (flags & 0x04) != 0;
    }
}

pub fn init() {
    unsafe {
        let mut cmd = Port::<u8>::new(0x64);
        let mut data = Port::<u8>::new(0x60);

        // Enable auxiliary mouse device
        wait_write();
        cmd.write(0xA8);

        // Enable interrupts in command byte
        wait_write();
        cmd.write(0x20);
        wait_read();
        let mut status = data.read();
        status |= 0x02; // Enable IRQ 12
        status &= !0x20; // Enable mouse clock
        wait_write();
        cmd.write(0x60);
        wait_write();
        data.write(status);

        // Set default mouse settings
        mouse_write(0xF6);
        let _ = mouse_read(); // ACK

        // Enable streaming
        mouse_write(0xF4);
        let _ = mouse_read(); // ACK
    }
    crate::serial_println!("[drivers] PS/2 Mouse initialized");
}

fn wait_write() {
    let mut cmd = Port::<u8>::new(0x64);
    for _ in 0..100_000 {
        if (unsafe { cmd.read() } & 0x02) == 0 {
            return;
        }
    }
}

fn wait_read() {
    let mut cmd = Port::<u8>::new(0x64);
    for _ in 0..100_000 {
        if (unsafe { cmd.read() } & 0x01) != 0 {
            return;
        }
    }
}

fn mouse_write(val: u8) {
    let mut cmd = Port::<u8>::new(0x64);
    let mut data = Port::<u8>::new(0x60);
    wait_write();
    unsafe {
        cmd.write(0xD4);
        wait_write();
        data.write(val);
    }
}

fn mouse_read() -> u8 {
    wait_read();
    let mut data = Port::<u8>::new(0x60);
    unsafe { data.read() }
}

pub fn on_interrupt() {
    let mut cmd = Port::<u8>::new(0x64);
    let mut data = Port::<u8>::new(0x60);
    let status = unsafe { cmd.read() };
    if (status & 0x21) == 0x21 {
        let byte = unsafe { data.read() };
        MOUSE.lock().handle_byte(byte);
    }
}

pub fn get_mouse_state() -> MouseState {
    MOUSE.lock().get_state()
}

pub fn set_screen_size(width: u32, height: u32) {
    MOUSE.lock().set_bounds(width, height);
}
