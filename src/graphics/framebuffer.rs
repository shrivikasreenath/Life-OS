use alloc::vec;
use alloc::vec::Vec;
use limine::request::FramebufferRequest;
use spin::Mutex;

#[used]
pub static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn hex(val: u32) -> Self {
        Self {
            r: ((val >> 16) & 0xFF) as u8,
            g: ((val >> 8) & 0xFF) as u8,
            b: (val & 0xFF) as u8,
            a: 255,
        }
    }

    pub const fn to_u32(self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    pub fn blend(self, bg: Color) -> Color {
        if self.a == 255 {
            return self;
        }
        if self.a == 0 {
            return bg;
        }

        let alpha = self.a as u32;
        let inv_alpha = 255 - alpha;

        Color {
            r: ((self.r as u32 * alpha + bg.r as u32 * inv_alpha) / 255) as u8,
            g: ((self.g as u32 * alpha + bg.g as u32 * inv_alpha) / 255) as u8,
            b: ((self.b as u32 * alpha + bg.b as u32 * inv_alpha) / 255) as u8,
            a: 255,
        }
    }
}

pub struct Framebuffer {
    hw_ptr: *mut u8,
    pub width: usize,
    pub height: usize,
    pub pitch: usize,
    pub bpp: usize,
    pub backbuffer: Vec<u32>,
}

unsafe impl Send for Framebuffer {}
unsafe impl Sync for Framebuffer {}

impl Framebuffer {
    pub fn new() -> Option<Self> {
        let response = FRAMEBUFFER_REQUEST.get_response()?;
        let fb = response.framebuffers().next()?;

        let width = fb.width() as usize;
        let height = fb.height() as usize;
        let pitch = fb.pitch() as usize;
        let bpp = fb.bpp() as usize;
        let hw_ptr = fb.addr();

        crate::serial_println!(
            "[gfx] Framebuffer acquired: {}x{} @ {} bpp (pitch: {} bytes)",
            width,
            height,
            bpp,
            pitch
        );

        let backbuffer = vec![0u32; width * height];

        Some(Self {
            hw_ptr,
            width,
            height,
            pitch,
            bpp,
            backbuffer,
        })
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            if color.a == 255 {
                self.backbuffer[idx] = color.to_u32();
            } else {
                let current = Color::hex(self.backbuffer[idx]);
                self.backbuffer[idx] = color.blend(current).to_u32();
            }
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Color {
        if x < self.width && y < self.height {
            Color::hex(self.backbuffer[y * self.width + x])
        } else {
            Color::rgb(0, 0, 0)
        }
    }

    pub fn clear(&mut self, color: Color) {
        let val = color.to_u32();
        self.backbuffer.fill(val);
    }

    pub fn present(&self) {
        unsafe {
            let bytes_per_pixel = self.bpp / 8;
            for y in 0..self.height {
                let src_offset = y * self.width;
                let dst_offset = y * self.pitch;

                let src_slice = &self.backbuffer[src_offset..src_offset + self.width];
                let dst_ptr = self.hw_ptr.add(dst_offset) as *mut u32;

                if bytes_per_pixel == 4 {
                    core::ptr::copy_nonoverlapping(src_slice.as_ptr(), dst_ptr, self.width);
                } else {
                    // Fallback for other bpp if needed
                    for x in 0..self.width {
                        let c = src_slice[x];
                        let pixel_ptr = self.hw_ptr.add(dst_offset + x * bytes_per_pixel);
                        *pixel_ptr = (c & 0xFF) as u8;
                        *pixel_ptr.add(1) = ((c >> 8) & 0xFF) as u8;
                        *pixel_ptr.add(2) = ((c >> 16) & 0xFF) as u8;
                    }
                }
            }
        }
    }
}

pub static GFX: Mutex<Option<Framebuffer>> = Mutex::new(None);

pub fn init() -> bool {
    if let Some(fb) = Framebuffer::new() {
        crate::drivers::mouse::set_screen_size(fb.width as u32, fb.height as u32);
        *GFX.lock() = Some(fb);
        true
    } else {
        crate::serial_println!("[gfx] No graphical framebuffer found");
        false
    }
}

pub fn get_dimensions() -> (usize, usize) {
    if let Some(ref fb) = *GFX.lock() {
        (fb.width, fb.height)
    } else {
        (1024, 768)
    }
}
