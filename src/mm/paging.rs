use core::sync::atomic::{AtomicU64, Ordering};
use limine::request::HhdmRequest;

#[used]
pub static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

static HHDM_OFFSET: AtomicU64 = AtomicU64::new(0);

pub fn init() {
    if let Some(hhdm_response) = HHDM_REQUEST.get_response() {
        let offset = hhdm_response.offset();
        HHDM_OFFSET.store(offset, Ordering::SeqCst);
        crate::serial_println!("[mm] HHDM offset: 0x{:016x}", offset);
    } else {
        panic!("Limine HHDM request failed");
    }
}

pub fn get_hhdm_offset() -> u64 {
    HHDM_OFFSET.load(Ordering::SeqCst)
}

#[inline]
pub fn phys_to_virt(phys: u64) -> u64 {
    phys + get_hhdm_offset()
}

#[inline]
pub fn virt_to_phys(virt: u64) -> u64 {
    virt - get_hhdm_offset()
}
