use log::warn;

#[allow(clippy::upper_case_acronyms)]
pub struct RAM {
    wram_bank: usize,
    wram: [[u8; 0x1000]; 8],
    hram: [u8; 0x80],
}

impl RAM {
    pub fn new() -> Self {
        Self {
            wram_bank: 1,
            wram: [[0; 0x1000]; 8],
            hram: [0; 0x80],
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0xc000..=0xcfff => self.wram[0][(addr - 0xc000) as usize] = value,
            0xd000..=0xdfff => self.wram[self.wram_bank][(addr - 0xd000) as usize] = value,
            0xff70 => self.wram_bank = (value & 0b111).max(1) as usize,
            0xff80..=0xfffe => self.hram[(addr - 0xff80) as usize] = value,
            _ => warn!("ram.write: unhandled address 0x{addr:04X}"),
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xc000..=0xcfff => self.wram[0][(addr - 0xc000) as usize],
            0xd000..=0xdfff => self.wram[self.wram_bank][(addr - 0xd000) as usize],
            0xff70 => self.wram_bank as u8,
            0xff80..=0xfffe => self.hram[(addr - 0xff80) as usize],
            _ => {
                warn!("ram.read: unhandled address 0x{addr:04X}");
                0xff
            }
        }
    }
}
