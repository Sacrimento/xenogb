use crate::utils::{get_bit, set_bit};

enum SerialTransferControlFlags {
    CLOCK_SELECT = 0,
    CLOCK_SPEED = 1,
    TRANSFER_ENABLE = 7,
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
            _ => panic!("Invalid addr for serial.write")
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xff01 => self.transfer_data,
            0xff02 => self.transfer_control,
            _ => panic!("Invalid addr for serial.read")
        }
    }

    pub fn get_char(&mut self) -> u8 {
        if get_bit(self.transfer_control, SerialTransferControlFlags::TRANSFER_ENABLE as u8) == 1 {
            self.transfer_control = set_bit(self.transfer_control, SerialTransferControlFlags::TRANSFER_ENABLE as u8, 0);
            return self.transfer_data;
        }
        0
    }
}