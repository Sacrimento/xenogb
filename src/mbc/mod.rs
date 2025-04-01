mod mbc1;
mod mbc5;

use mbc1::MBC1;
use mbc5::MBC5;

pub trait MemoryBankController {
    fn read(&self, addr: u16) -> u8;

    fn write(&mut self, addr: u16, value: u8) -> ();

    fn init_sram(ram_code: u8) -> Vec<[u8; 0x2000]>
    where
        Self: Sized,
    {
        let banks = match ram_code {
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
}

pub fn mbc(
    mbc_code: u8,
    ram_code: u8,
    rom: Vec<u8>,
) -> Box<dyn MemoryBankController + Send + Sync> {
    match mbc_code {
        0 => Box::new(NoMBC::new(rom)),
        1 => Box::new(MBC1::new(rom, ram_code)),
        2 => Box::new(MBC1::new(rom, ram_code)),
        3 => Box::new(MBC1::new(rom, ram_code)),
        0x1b => Box::new(MBC5::new(rom, ram_code)),
        _ => todo!(),
    }
}
