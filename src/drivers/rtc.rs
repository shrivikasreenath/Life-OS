use x86_64::instructions::port::Port;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl DateTime {
    pub const fn zero() -> Self {
        Self {
            year: 2026,
            month: 1,
            day: 1,
            hour: 0,
            minute: 0,
            second: 0,
        }
    }
}

pub fn read_rtc() -> DateTime {
    unsafe {
        // Wait until RTC update is not in progress
        while is_updating() {}

        let mut sec = read_cmos(0x00);
        let mut min = read_cmos(0x02);
        let mut hour = read_cmos(0x04);
        let mut day = read_cmos(0x07);
        let mut month = read_cmos(0x08);
        let mut year = read_cmos(0x09);
        let register_b = read_cmos(0x0B);

        // Convert BCD to binary if bit 2 of Register B is 0
        if (register_b & 0x04) == 0 {
            sec = (sec & 0x0F) + ((sec / 16) * 10);
            min = (min & 0x0F) + ((min / 16) * 10);
            hour = ((hour & 0x0F) + (((hour & 0x70) / 16) * 10)) | (hour & 0x80);
            day = (day & 0x0F) + ((day / 16) * 10);
            month = (month & 0x0F) + ((month / 16) * 10);
            year = (year & 0x0F) + ((year / 16) * 10);
        }

        // Convert 12 hour to 24 hour if necessary
        if (register_b & 0x02) == 0 && (hour & 0x80) != 0 {
            hour = ((hour & 0x7F) + 12) % 24;
        }

        let full_year = 2000 + (year as u16);

        DateTime {
            year: full_year,
            month,
            day,
            hour,
            minute: min,
            second: sec,
        }
    }
}

unsafe fn is_updating() -> bool {
    let mut reg_port = Port::<u8>::new(0x70);
    let mut data_port = Port::<u8>::new(0x71);
    reg_port.write(0x0A);
    (data_port.read() & 0x80) != 0
}

unsafe fn read_cmos(reg: u8) -> u8 {
    let mut reg_port = Port::<u8>::new(0x70);
    let mut data_port = Port::<u8>::new(0x71);
    reg_port.write(reg);
    data_port.read()
}
