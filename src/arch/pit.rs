use core::sync::atomic::{AtomicU64, Ordering};
use x86_64::instructions::port::Port;

static TICKS: AtomicU64 = AtomicU64::new(0);

pub const PIT_FREQUENCY_HZ: u64 = 100;

pub fn init() {
    let divisor: u16 = (1193182 / PIT_FREQUENCY_HZ) as u16;

    unsafe {
        let mut cmd_port = Port::<u8>::new(0x43);
        let mut data_port = Port::<u8>::new(0x40);

        // Channel 0, lobyte/hibyte, rate generator (Mode 2)
        cmd_port.write(0x34);
        data_port.write((divisor & 0xFF) as u8);
        data_port.write(((divisor >> 8) & 0xFF) as u8);
    }
}

pub fn on_tick() {
    TICKS.fetch_add(1, Ordering::Relaxed);
}

pub fn get_ticks() -> u64 {
    TICKS.load(Ordering::Relaxed)
}

pub fn uptime_seconds() -> u64 {
    get_ticks() / PIT_FREQUENCY_HZ
}

pub fn uptime_ms() -> u64 {
    (get_ticks() * 1000) / PIT_FREQUENCY_HZ
}

pub fn sleep_ms(ms: u64) {
    if ms == 0 {
        return;
    }
    let target = uptime_ms().saturating_add(ms);
    let mut spins = 0u64;
    while uptime_ms() < target {
        core::hint::spin_loop();
        spins += 1;
        // Safety timeout (~10,000,000 spins per 10ms on modern CPUs) in case timer interrupt is blocked
        if spins > ms.saturating_mul(1_000_000) {
            break;
        }
    }
}
