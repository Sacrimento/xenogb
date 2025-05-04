use std::path::PathBuf;

use log::warn;

use super::MemoryBankController;

pub struct MBC1 {
    rom: Vec<u8>,
    sram: Vec<[u8; 0x2000]>,
    rom_bank: usize,
    rom_bank_mask: usize,
    ram_bank: usize,
    ram_enable: bool,
    banking_mode: u8,

    has_save: bool,
    save_fname: PathBuf,
}

impl MBC1 {
    pub fn new(
        rom: Vec<u8>,
        ram_banks_code: u8,
        rom_banks_code: u8,
        has_save: bool,
        mut rom_fname: PathBuf,
    ) -> Self {
        rom_fname.set_extension("gbsave");

        Self {
            rom,
            rom_bank: 1,
            rom_bank_mask: ((1 << (1 + rom_banks_code)) - 1) as usize,
            ram_bank: 0,
            ram_enable: false,
            banking_mode: 0,
            sram: Self::load_sram(&rom_fname, has_save, ram_banks_code),
            has_save,
            save_fname: rom_fname,
        }
    }
}

impl MemoryBankController for MBC1 {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0..=0x3fff => self.rom[addr as usize],
            0x4000..=0x7fff => {
                let offset = if self.banking_mode == 1 {
                    ((self.ram_bank << 5) | self.rom_bank) * 0x4000
                } else {
                    self.rom_bank * 0x4000
                };
                self.rom[offset + (addr as usize - 0x4000)]
            }
            0xa000..=0xbfff => {
                if !self.ram_enable {
                    return 0xff;
                }
                self.sram[self.ram_bank][addr as usize - 0xa000]
            }
            _ => {
                warn!("mbc1.read: unhandled address 0x{addr:04X}");
                0xff
            }
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0..=0x1fff => self.ram_enable = value & 0xf == 0xa,
            0x2000..=0x3fff => self.rom_bank = (value as usize & 0x1f).max(1) & self.rom_bank_mask,
            0x4000..=0x5fff => self.ram_bank = value as usize & 0x3,
            0x6000..=0x7fff => self.banking_mode = value & 0x1,
            0xa000..=0xbfff => {
                if self.ram_enable {
                    self.sram[self.ram_bank][addr as usize - 0xa000] = value;
                }
            }
            _ => warn!("mbc1.write: unhandled address 0x{addr:04X}"),
        }
    }

    fn save(&self) {
        if self.has_save {
            Self::save_sram(&self.save_fname, &self.sram);
        }
    }
}
