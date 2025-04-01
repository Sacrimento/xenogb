use crate::between;

use super::MemoryBankController;

pub struct MBC5 {
    rom: Vec<u8>,
    sram: Vec<[u8; 0x2000]>,
    rom_bank: usize,
    ram_bank: usize,
    ram_enable: bool,
    banking_mode: u8,
}

impl MBC5 {
    pub fn new(rom: Vec<u8>, ram_code: u8) -> Self {
        Self {
            rom,
            rom_bank: 0,
            ram_bank: 0,
            ram_enable: false,
            banking_mode: 0,
            sram: <MBC5 as MemoryBankController>::init_sram(ram_code),
        }
    }
}

impl MemoryBankController for MBC5 {
    fn read(&self, addr: u16) -> u8 {
        if between!(addr, 0x0, 0x3fff) {
            return self.rom[addr as usize];
        } else if between!(addr, 0x4000, 0x7fff) {
            let offset = self.rom_bank * 0x4000;
            return self.rom[offset + (addr as usize - 0x4000)];
        } else if between!(addr, 0xa000, 0xbfff) {
            if !self.ram_enable {
                return 0xff;
            }
            return self.sram[self.ram_bank][addr as usize - 0xa000];
        }
        panic!("Invalid addr for mbc5.read");
    }

    fn write(&mut self, addr: u16, value: u8) {
        if between!(addr, 0x0, 0x1fff) {
            self.ram_enable = value & 0xf == 0xa;
        } else if between!(addr, 0x2000, 0x2fff) {
            self.rom_bank = (self.rom_bank & 0x100) | value as usize & 0xff;
        } else if between!(addr, 0x3000, 0x3fff) {
            self.rom_bank = (self.rom_bank & 0xff) | ((value as usize & 0x1) << 8);
        } else if between!(addr, 0x4000, 0x5fff) {
            self.ram_bank = value as usize & 0xf;
        } else if between!(addr, 0xa000, 0xbfff) {
            if self.ram_enable {
                self.sram[self.ram_bank][addr as usize - 0xa000] = value;
            }
        } else {
            println!("Unhandled mbc5.write at {:04X}", addr);
        }
    }
}
