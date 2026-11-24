pub mod canvas;
pub mod font;
pub mod framebuffer;
pub mod icons;

pub fn init() -> bool {
    framebuffer::init()
}
