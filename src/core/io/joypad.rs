use crate::flag_set;

const PAD: u8 = 0x10;
const ACTION: u8 = 0x20;

#[allow(nonstandard_style)]
pub mod JOYPAD_INPUT {
    pub const A: u8 = 0x1;
    pub const B: u8 = 0x2;
    pub const START: u8 = 0x4;
    pub const SELECT: u8 = 0x8;
    pub const RIGHT: u8 = 0x10;
    pub const LEFT: u8 = 0x20;
    pub const UP: u8 = 0x40;
    pub const DOWN: u8 = 0x80;
}

pub struct Joypad {
    state: u8,
    selector: u8,
}

#[allow(clippy::new_without_default)]
impl Joypad {
    pub fn new() -> Self {
        Self {
            state: 0xff,
            selector: PAD | ACTION,
        }
    }

    pub fn write(&mut self, value: u8) {
        self.selector = value & 0xf0;
    }

    pub fn read(&self) -> u8 {
        if flag_set!(self.selector, ACTION | PAD) {
            return 0xf;
        }

        match self.selector & (ACTION | PAD) {
            PAD => self.state & 0xf,
            ACTION => self.state >> 4,
            _ => unreachable!(),
        }
    }

    pub fn press(&mut self, button: u8) {
        self.state &= !button;
    }

    pub fn release(&mut self, button: u8) {
        self.state |= button;
    }
}
