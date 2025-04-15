mod mbc1;
mod mbc3;
mod mbc5;
use std::io::Write;
use std::{
    fs::{self, File},
    path::PathBuf,
};

use log::info;
use mbc1::MBC1;
use mbc3::MBC3;
use mbc5::MBC5;

pub trait MemoryBankController {
    fn read(&self, addr: u16) -> u8;

    fn write(&mut self, addr: u16, value: u8) -> ();

    fn save(&self) -> ();

    fn build_sram(ram_banks_code: u8) -> Vec<[u8; 0x2000]>
    where
        Self: Sized,
    {
        let banks = match ram_banks_code {
            0 | 1 => 0,
            2 => 1,
            3 => 4,
            4 => 16,
            5 => 8,
            _ => unreachable!(),
        };

        let mut vec = vec![];

        for _ in 0..banks {
            vec.push([0; 0x2000]);
        }
        vec
    }

    fn save_sram(save_fname: &PathBuf, sram: &Vec<[u8; 0x2000]>)
    where
        Self: Sized,
    {
        info!("SRAM saved!");
        let mut file = File::create(save_fname).unwrap();
        for bank in sram.iter() {
            file.write(&bank[..]).unwrap();
        }
    }

    fn load_sram(path: &PathBuf, has_save: bool, ram_banks_code: u8) -> Vec<[u8; 0x2000]>
    where
        Self: Sized,
    {
        if !has_save || !path.exists() {
            return Self::build_sram(ram_banks_code);
        }

        let bytes = fs::read(path).unwrap();
        let mut sram: Vec<[u8; 0x2000]> = Vec::new();
        sram.resize(bytes.len() / 0x2000, [0; 0x2000]);

        for bank in 0..sram.len() {
            sram[bank].copy_from_slice(&bytes[bank * 0x2000..bank * 0x2000 + 0x2000]);
        }
        sram
    }
}

struct NoMBC {
    rom: Vec<u8>,
}

impl NoMBC {
    pub fn new(rom: Vec<u8>) -> Self {
        Self { rom }
    }
}

impl MemoryBankController for NoMBC {
    fn write(&mut self, _: u16, _: u8) -> () {}

    fn read(&self, addr: u16) -> u8 {
        self.rom[addr as usize]
    }

    fn save(&self) {}
}

pub fn mbc(
    mbc_code: u8,
    ram_banks_code: u8,
    rom_banks_code: u8,
    rom: Vec<u8>,
    rom_fname: PathBuf,
) -> Box<dyn MemoryBankController + Send + Sync> {
    match mbc_code {
        0x0 => Box::new(NoMBC::new(rom)),
        0x1 | 0x2 => Box::new(MBC1::new(
            rom,
            ram_banks_code,
            rom_banks_code,
            false,
            rom_fname,
        )),
        0x3 => Box::new(MBC1::new(
            rom,
            ram_banks_code,
            rom_banks_code,
            true,
            rom_fname,
        )),
        0xf | 0x10 => Box::new(MBC3::new(rom, ram_banks_code, true, true, rom_fname)),
        0x11 | 0x12 => Box::new(MBC3::new(rom, ram_banks_code, false, false, rom_fname)),
        0x13 => Box::new(MBC3::new(rom, ram_banks_code, true, false, rom_fname)),
        0x19 | 0x1a | 0x1c | 0x1d => Box::new(MBC5::new(rom, ram_banks_code, false, rom_fname)),
        0x1b | 0x1e => Box::new(MBC5::new(rom, ram_banks_code, true, rom_fname)),
        _ => todo!(),
    }
}
