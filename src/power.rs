use x86_64::instructions::port::Port;

pub fn shutdown() -> ! {
    crate::serial_println!("[power] Initiating Life OS shutdown sequence...");

    unsafe {
        // QEMU / VirtualBox ACPI shutdown (port 0x604)
        let mut port_604 = Port::<u16>::new(0x604);
        port_604.write(0x2000);

        // Older QEMU / Bochs shutdown (port 0xB004)
        let mut port_b004 = Port::<u16>::new(0xB004);
        port_b004.write(0x2000);

        // VirtualBox power off debug port
        let mut port_vbox = Port::<u32>::new(0x4004);
        port_vbox.write(0x3400);
    }

    crate::serial_println!("[power] System halted. You may safely turn off the virtual machine.");
    loop {
        x86_64::instructions::hlt();
    }
}

pub fn reboot() -> ! {
    crate::serial_println!("[power] Initiating Life OS reboot sequence...");

    unsafe {
        // 8042 PS/2 controller pulse reset line (port 0x64)
        let mut port_64 = Port::<u8>::new(0x64);
        // Wait until input buffer empty
        for _ in 0..100_000 {
            if (port_64.read() & 0x02) == 0 {
                break;
            }
        }
        port_64.write(0xFE);

        // Fallback: Trigger triple fault via empty IDT
        let null_idt = x86_64::structures::DescriptorTablePointer {
            limit: 0,
            base: x86_64::VirtAddr::new(0),
        };
        x86_64::instructions::tables::lidt(&null_idt);
        core::arch::asm!("int3");
    }

    loop {
        x86_64::instructions::hlt();
    }
}
