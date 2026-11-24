use alloc::vec::Vec;
use x86_64::instructions::port::Port;

const PCI_CONFIG_ADDRESS: u16 = 0xCF8;
const PCI_CONFIG_DATA: u16 = 0xCFC;

#[derive(Debug, Clone, Copy)]
pub struct PciDevice {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class_id: u8,
    pub subclass_id: u8,
    pub bar0: u32,
    pub interrupt_line: u8,
}

pub fn read_pci_u32(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    let address = ((bus as u32) << 16)
        | ((device as u32) << 11)
        | ((function as u32) << 8)
        | ((offset as u32) & 0xFC)
        | 0x80000000;

    unsafe {
        let mut addr_port = Port::<u32>::new(PCI_CONFIG_ADDRESS);
        let mut data_port = Port::<u32>::new(PCI_CONFIG_DATA);
        addr_port.write(address);
        data_port.read()
    }
}

pub fn write_pci_u32(bus: u8, device: u8, function: u8, offset: u8, value: u32) {
    let address = ((bus as u32) << 16)
        | ((device as u32) << 11)
        | ((function as u32) << 8)
        | ((offset as u32) & 0xFC)
        | 0x80000000;

    unsafe {
        let mut addr_port = Port::<u32>::new(PCI_CONFIG_ADDRESS);
        let mut data_port = Port::<u32>::new(PCI_CONFIG_DATA);
        addr_port.write(address);
        data_port.write(value);
    }
}

pub fn read_pci_u16(bus: u8, device: u8, function: u8, offset: u8) -> u16 {
    let val = read_pci_u32(bus, device, function, offset);
    if (offset & 2) != 0 {
        (val >> 16) as u16
    } else {
        (val & 0xFFFF) as u16
    }
}

pub fn scan_pci() -> Vec<PciDevice> {
    let mut devices = Vec::new();

    for bus in 0..=32 {
        for device in 0..32 {
            let vendor_id = read_pci_u16(bus, device, 0, 0x00);
            if vendor_id == 0xFFFF {
                continue;
            }

            let device_id = read_pci_u16(bus, device, 0, 0x02);
            let class_subclass = read_pci_u16(bus, device, 0, 0x0A);
            let class_id = (class_subclass >> 8) as u8;
            let subclass_id = (class_subclass & 0xFF) as u8;
            let bar0 = read_pci_u32(bus, device, 0, 0x10);
            let interrupt_info = read_pci_u16(bus, device, 0, 0x3C);
            let interrupt_line = (interrupt_info & 0xFF) as u8;

            devices.push(PciDevice {
                bus,
                device,
                function: 0,
                vendor_id,
                device_id,
                class_id,
                subclass_id,
                bar0,
                interrupt_line,
            });
        }
    }

    devices
}

pub fn enable_bus_master(dev: &PciDevice) {
    let cmd = read_pci_u32(dev.bus, dev.device, dev.function, 0x04);
    // Bit 0: I/O enable, Bit 1: Memory enable, Bit 2: Bus master enable
    write_pci_u32(dev.bus, dev.device, dev.function, 0x04, cmd | 0x07);
}
