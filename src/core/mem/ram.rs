use log::warn;

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
        match addr {
            0xc000..=0xdfff => self.wram[(addr - 0xc000) as usize] = value,
            0xff80..=0xfffe => self.hram[(addr - 0xff80) as usize] = value,
            _ => warn!("ram.write: unhandled address 0x{addr:04X}"),
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xc000..=0xdfff => self.wram[(addr - 0xc000) as usize],
            0xff80..=0xfffe => self.hram[(addr - 0xff80) as usize],
            _ => {
                warn!("ram.read: unhandled address 0x{addr:04X}");
                0xff
            }
        }
    }
}
