use crate::between;

pub struct RAM {
    wram: [u8; 0x2000],
    hram: [u8; 0x80],
}

impl RAM {
    pub fn new() -> Self {
        Self {
            wram: [0; 0x2000],
            hram: [0; 0x80],
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        if between!(addr, 0xc000, 0xdfff) {
            self.wram[(addr - 0xc000) as usize] = value;
        } else if between!(addr, 0xff80, 0xfffe) {
            self.hram[(addr - 0xff80) as usize] = value;
        } else {
            panic!("Invalid addr 0x{:02x} for ram.write", addr);
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        if between!(addr, 0xc000, 0xdfff) {
            return self.wram[(addr - 0xc000) as usize]
        } else if between!(addr, 0xff80, 0xfffe) {
            return self.hram[(addr - 0xff80) as usize]
        }

        panic!("Invalid addr 0x{:02x} for ram.read", addr);
    }
}