use crate::flag_set;

#[allow(nonstandard_style)]
mod SerialTransferControlFlags {
    pub const _CLOCK_SELECT: u8 = 0x1;
    pub const _CLOCK_SPEED: u8 = 0x2;
    pub const TRANSFER_ENABLE: u8 = 0x80;
}

#[derive(Default)]
pub struct Serial {
    transfer_control: u8,
    transfer_data: u8,
}

impl Serial {
    pub fn write(&mut self, addr: u16, value: u8) -> () {
        match addr {
            0xff01 => self.transfer_data = value,
            0xff02 => self.transfer_control = value,
            _ => unreachable!(),
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xff01 => self.transfer_data,
            0xff02 => self.transfer_control,
            _ => unreachable!(),
        }
    }

    pub fn get_char(&mut self) -> u8 {
        if flag_set!(
            self.transfer_control,
            SerialTransferControlFlags::TRANSFER_ENABLE
        ) {
            self.transfer_control =
                self.transfer_control & !(SerialTransferControlFlags::TRANSFER_ENABLE);
            return self.transfer_data;
        }
        0
    }
}
