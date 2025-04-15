use log::warn;
use std::path::PathBuf;

use super::MemoryBankController;

pub struct MBC5 {
    rom: Vec<u8>,
    sram: Vec<[u8; 0x2000]>,
    rom_bank: usize,
    ram_bank: usize,
    ram_enable: bool,

    has_save: bool,
    save_fname: PathBuf,
}

impl MBC5 {
    pub fn new(rom: Vec<u8>, ram_banks_code: u8, has_save: bool, mut rom_fname: PathBuf) -> Self {
        rom_fname.set_extension("gbsave");

        Self {
            rom,
            rom_bank: 0,
            ram_bank: 0,
            ram_enable: false,
            sram: Self::load_sram(&rom_fname, has_save, ram_banks_code),
            has_save,
            save_fname: rom_fname,
        }
    }
}

impl MemoryBankController for MBC5 {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0..=0x3fff => self.rom[addr as usize],
            0x4000..=0x7fff => self.rom[(self.rom_bank * 0x4000) + (addr as usize - 0x4000)],
            0xa000..=0xbfff => {
                if !self.ram_enable {
                    return 0xff;
                }
                self.sram[self.ram_bank][addr as usize - 0xa000]
            }
            _ => {
                warn!("mbc5.read: unhandled address 0x{addr:04X}");
                0xff
            }
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0..=0x1fff => self.ram_enable = value & 0xf == 0xa,
            0x2000..=0x2fff => self.rom_bank = (self.rom_bank & 0x100) | value as usize & 0xff,
            0x3000..=0x3fff => {
                self.rom_bank = (self.rom_bank & 0xff) | ((value as usize & 0x1) << 8)
            }
            0x4000..=0x5fff => self.ram_bank = value as usize & 0xf,
            0xa000..=0xbfff => {
                if self.ram_enable {
                    self.sram[self.ram_bank][addr as usize - 0xa000] = value;
                }
            }
            _ => warn!("mbc5.write: unhandled address 0x{addr:04X}"),
        }
    }

    fn save(&self) {
        if self.has_save {
            Self::save_sram(&self.save_fname, &self.sram);
        }
    }
}
