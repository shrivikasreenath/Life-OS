use crate::drivers::pci::{enable_bus_master, scan_pci};
use crate::mm::frame::alloc_frame;
use crate::mm::paging::phys_to_virt;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;

const E1000_NUM_RX_DESC: usize = 32;
const E1000_NUM_TX_DESC: usize = 32;
const E1000_BUFFER_SIZE: usize = 2048;

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct RxDesc {
    address: u64,
    length: u16,
    checksum: u16,
    status: u8,
    errors: u8,
    special: u16,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct TxDesc {
    address: u64,
    length: u16,
    cso: u8,
    cmd: u8,
    status: u8,
    css: u8,
    special: u16,
}

pub struct E1000Driver {
    mmio_base: u64,
    mac: [u8; 6],
    rx_descs_virt: *mut RxDesc,
    rx_buffers_virt: [*mut u8; E1000_NUM_RX_DESC],
    rx_cur: usize,
    tx_descs_virt: *mut TxDesc,
    tx_buffers_virt: [*mut u8; E1000_NUM_TX_DESC],
    tx_cur: usize,
    packets_rx: AtomicU64,
    packets_tx: AtomicU64,
    link_up: bool,
}

unsafe impl Send for E1000Driver {}
unsafe impl Sync for E1000Driver {}

pub static E1000: Mutex<Option<E1000Driver>> = Mutex::new(None);

impl E1000Driver {
    unsafe fn write_reg(&self, reg: u32, val: u32) {
        let ptr = (self.mmio_base + reg as u64) as *mut u32;
        core::ptr::write_volatile(ptr, val);
    }

    unsafe fn read_reg(&self, reg: u32) -> u32 {
        let ptr = (self.mmio_base + reg as u64) as *const u32;
        core::ptr::read_volatile(ptr)
    }

    pub fn mac_address(&self) -> [u8; 6] {
        self.mac
    }

    pub fn is_link_up(&self) -> bool {
        self.link_up
    }

    pub fn stats(&self) -> (u64, u64) {
        (
            self.packets_rx.load(Ordering::Relaxed),
            self.packets_tx.load(Ordering::Relaxed),
        )
    }

    pub fn send(&mut self, packet: &[u8]) -> bool {
        if packet.len() > E1000_BUFFER_SIZE {
            return false;
        }

        let desc_ptr = unsafe { self.tx_descs_virt.add(self.tx_cur) };
        let buf_ptr = self.tx_buffers_virt[self.tx_cur];

        unsafe {
            core::ptr::copy_nonoverlapping(packet.as_ptr(), buf_ptr, packet.len());
            // cmd: bit 0 = EOP (End of Packet), bit 3 = RS (Report Status)
            (*desc_ptr).length = packet.len() as u16;
            (*desc_ptr).cmd = (1 << 0) | (1 << 3);
            (*desc_ptr).status = 0;

            let _prev_cur = self.tx_cur;
            self.tx_cur = (self.tx_cur + 1) % E1000_NUM_TX_DESC;
            self.write_reg(0x3818, self.tx_cur as u32); // TDT

            // Wait for transmission completion
            let mut timeout = 100_000;
            while ((*desc_ptr).status & 0x01) == 0 && timeout > 0 {
                core::hint::spin_loop();
                timeout -= 1;
            }

            self.packets_tx.fetch_add(1, Ordering::Relaxed);
        }

        true
    }

    pub fn receive(&mut self) -> Option<Vec<u8>> {
        let next = (self.rx_cur + 1) % E1000_NUM_RX_DESC;
        let desc_ptr = unsafe { self.rx_descs_virt.add(next) };

        unsafe {
            // Check DD bit (bit 0 of status)
            if ((*desc_ptr).status & 0x01) != 0 {
                let len = (*desc_ptr).length as usize;
                let buf_ptr = self.rx_buffers_virt[next];

                let mut packet = Vec::with_capacity(len);
                packet.set_len(len);
                core::ptr::copy_nonoverlapping(buf_ptr, packet.as_mut_ptr(), len);

                (*desc_ptr).status = 0;
                self.rx_cur = next;
                self.write_reg(0x2818, self.rx_cur as u32); // RDT

                self.packets_rx.fetch_add(1, Ordering::Relaxed);
                return Some(packet);
            }
        }

        None
    }
}

pub fn init() -> bool {
    let devices = scan_pci();
    let mut e1000_pci = None;

    for dev in &devices {
        // Intel Vendor ID = 0x8086
        // Device IDs: 0x100E (82540EM - VirtualBox / QEMU default), 0x1004, 0x100F, 0x153A, 0x10D3
        if dev.vendor_id == 0x8086
            && (dev.device_id == 0x100E
                || dev.device_id == 0x1004
                || dev.device_id == 0x100F
                || dev.device_id == 0x10D3
                || dev.device_id == 0x153A)
        {
            e1000_pci = Some(*dev);
            break;
        }
    }

    let dev = match e1000_pci {
        Some(d) => d,
        None => {
            crate::serial_println!("[e1000] Intel 82540EM NIC not detected on PCI bus");
            return false;
        }
    };

    enable_bus_master(&dev);

    let mmio_phys = (dev.bar0 & 0xFFFFFFF0) as u64;
    let mmio_virt = phys_to_virt(mmio_phys);

    crate::serial_println!(
        "[e1000] Found Intel NIC (Device ID: 0x{:04x}) at MMIO phys: 0x{:08x}",
        dev.device_id,
        mmio_phys
    );

    // Read MAC address from RAL/RAH
    let mut mac = [0u8; 6];
    unsafe {
        let ral = core::ptr::read_volatile((mmio_virt + 0x5400) as *const u32);
        let rah = core::ptr::read_volatile((mmio_virt + 0x5404) as *const u32);
        mac[0] = (ral & 0xFF) as u8;
        mac[1] = ((ral >> 8) & 0xFF) as u8;
        mac[2] = ((ral >> 16) & 0xFF) as u8;
        mac[3] = ((ral >> 24) & 0xFF) as u8;
        mac[4] = (rah & 0xFF) as u8;
        mac[5] = ((rah >> 8) & 0xFF) as u8;
    }

    crate::serial_println!(
        "[e1000] Hardware MAC Address: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
        mac[0],
        mac[1],
        mac[2],
        mac[3],
        mac[4],
        mac[5]
    );

    // Allocate DMA rings
    let rx_ring_phys = alloc_frame().expect("Alloc RX ring failed");
    let rx_ring_virt = phys_to_virt(rx_ring_phys) as *mut RxDesc;

    let tx_ring_phys = alloc_frame().expect("Alloc TX ring failed");
    let tx_ring_virt = phys_to_virt(tx_ring_phys) as *mut TxDesc;

    let mut rx_buffers = [core::ptr::null_mut(); E1000_NUM_RX_DESC];
    for i in 0..E1000_NUM_RX_DESC {
        let buf_phys = alloc_frame().expect("Alloc RX buffer failed");
        let buf_virt = phys_to_virt(buf_phys) as *mut u8;
        rx_buffers[i] = buf_virt;
        unsafe {
            let desc = rx_ring_virt.add(i);
            (*desc).address = buf_phys;
            (*desc).status = 0;
        }
    }

    let mut tx_buffers = [core::ptr::null_mut(); E1000_NUM_TX_DESC];
    for i in 0..E1000_NUM_TX_DESC {
        let buf_phys = alloc_frame().expect("Alloc TX buffer failed");
        let buf_virt = phys_to_virt(buf_phys) as *mut u8;
        tx_buffers[i] = buf_virt;
        unsafe {
            let desc = tx_ring_virt.add(i);
            (*desc).address = buf_phys;
            (*desc).status = 0;
            (*desc).cmd = 0;
        }
    }

    let mut driver = E1000Driver {
        mmio_base: mmio_virt,
        mac,
        rx_descs_virt: rx_ring_virt,
        rx_buffers_virt: rx_buffers,
        rx_cur: 0,
        tx_descs_virt: tx_ring_virt,
        tx_buffers_virt: tx_buffers,
        tx_cur: 0,
        packets_rx: AtomicU64::new(0),
        packets_tx: AtomicU64::new(0),
        link_up: false,
    };

    // Initialize hardware
    unsafe {
        // Device reset
        driver.write_reg(0x0000, 0x04000000);
        crate::arch::pit::sleep_ms(10);

        // Set Link Up, Auto-Speed Detection
        driver.write_reg(0x0000, (1 << 6) | (1 << 5) | (1 << 8));

        // Clear Multicast Table Array
        for i in 0..128 {
            driver.write_reg(0x5200 + i * 4, 0);
        }

        // Configure RX Ring
        driver.write_reg(0x2800, rx_ring_phys as u32);
        driver.write_reg(0x2804, (rx_ring_phys >> 32) as u32);
        driver.write_reg(
            0x2808,
            (E1000_NUM_RX_DESC * core::mem::size_of::<RxDesc>()) as u32,
        );
        driver.write_reg(0x2810, 0); // RDH
        driver.write_reg(0x2818, (E1000_NUM_RX_DESC - 1) as u32); // RDT

        // Configure RCTL: EN (bit 1), SBP (bit 2), UPE (bit 3), MPE (bit 4), BAM (bit 15), BSIZE 2048 (bits 16,17=0)
        driver.write_reg(0x0100, (1 << 1) | (1 << 2) | (1 << 3) | (1 << 4) | (1 << 15));

        // Configure TX Ring
        driver.write_reg(0x3800, tx_ring_phys as u32);
        driver.write_reg(0x3804, (tx_ring_phys >> 32) as u32);
        driver.write_reg(
            0x3808,
            (E1000_NUM_TX_DESC * core::mem::size_of::<TxDesc>()) as u32,
        );
        driver.write_reg(0x3810, 0); // TDH
        driver.write_reg(0x3818, 0); // TDT

        // Configure TCTL: EN (bit 1), PSP (bit 3), CT (0x0F << 4), COLD (0x40 << 12)
        driver.write_reg(0x0400, (1 << 1) | (1 << 3) | (0x0F << 4) | (0x40 << 12));

        // Check Link Status (bit 1 of STATUS register)
        let status = driver.read_reg(0x0008);
        driver.link_up = (status & 0x02) != 0;
    }

    crate::serial_println!(
        "[e1000] Driver initialization complete. Link Status: {}",
        if driver.link_up { "UP" } else { "DOWN" }
    );

    *E1000.lock() = Some(driver);
    true
}

pub fn send_packet(data: &[u8]) -> bool {
    if let Some(ref mut e1000) = *E1000.lock() {
        e1000.send(data)
    } else {
        false
    }
}

pub fn receive_packet() -> Option<Vec<u8>> {
    if let Some(ref mut e1000) = *E1000.lock() {
        e1000.receive()
    } else {
        None
    }
}

pub fn get_mac() -> [u8; 6] {
    if let Some(ref e1000) = *E1000.lock() {
        e1000.mac_address()
    } else {
        [0; 6]
    }
}

pub fn is_up() -> bool {
    if let Some(ref e1000) = *E1000.lock() {
        e1000.is_link_up()
    } else {
        false
    }
}

pub fn get_stats() -> (u64, u64) {
    if let Some(ref e1000) = *E1000.lock() {
        e1000.stats()
    } else {
        (0, 0)
    }
}
