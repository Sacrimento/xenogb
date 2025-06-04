use super::boot::{get_boot_rom, BootRom};
use super::cartridge::Cartridge;
use super::ram::RAM;
use crate::core::cpu::interrupts::{INTERRUPT_ENABLE, INTERRUPT_FLAGS};
use crate::core::io::video::ppu::Vbuf;
use crate::core::io::IOMMU;

use crossbeam_channel::Sender;
use log::{error, warn};

// 0x0000	0x3FFF	16 KiB ROM bank 00
// 0x4000	0x7FFF	16 KiB ROM Bank 01–NN
// 0x8000	0x9FFF	8 KiB Video RAM (VRAM)
// 0xA000	0xBFFF	8 KiB External RAM
// 0xC000	0xCFFF	4 KiB Work RAM (WRAM)
// 0xD000	0xDFFF	4 KiB Work RAM (WRAM)
// 0xE000	0xFDFF	Echo RAM (mirror of C000–DDFF)
// 0xFE00	0xFE9F	Object attribute memory (OAM)
// 0xFEA0	0xFEFF	Not Usable
// 0xFF00	0xFF7F	I/O Registers
// 0xFF80	0xFFFE	High RAM (HRAM)
// 0xFFFF	0xFFFF	Interrupt Enable register (IE)

#[derive(Clone)]
struct DMA {
    src: u16,
    dest: u16,
}

pub struct Bus {
    pub cartridge: Cartridge,
    ram: RAM,
    pub io: IOMMU,
    dma: Option<DMA>,

    pub booting: bool,
    boot_rom: &'static [u8; 0x100],
}

impl Bus {
    pub fn new(
        cartridge: Cartridge,
        boot_rom: BootRom,
        video_channel_sd: Sender<Vbuf>,
        audio_channel_sd: Sender<f32>,
    ) -> Self {
        Self {
            cartridge,
            ram: RAM::new(),
            io: IOMMU::new(video_channel_sd, audio_channel_sd),
            dma: None,
            booting: !matches!(boot_rom, BootRom::NONE),
            boot_rom: get_boot_rom(boot_rom),
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        if self.booting && addr <= 0x100 {
            return self.boot_rom[addr as usize];
        }

        match addr {
            0..=0x7fff => self.cartridge.read(addr),
            0x8000..=0x9fff => self.io.read(addr),
            0xa000..=0xbfff => self.cartridge.read(addr),
            0xc000..=0xdfff => self.ram.read(addr),
            0xe000..=0xfdff => self.ram.read(addr - 0x2000),
            0xfe00..=0xfe9f => self.io.read(addr),
            0xff0f => INTERRUPT_FLAGS.get(),
            0xff46 | 0xff51..=0xff55 => {
                warn!("Invalid DMA read at 0x{addr:04X}");
                0xff
            }
            0xff70 => self.ram.read(addr),
            0xff00..=0xff7f => self.io.read(addr),
            0xff80..=0xfffe => self.ram.read(addr),
            0xffff => INTERRUPT_ENABLE.get(),
            _ => {
                warn!("bus.read: unhandled address 0x{addr:04X}");
                0xff
            }
        }
    }

    pub fn read16(&self, addr: u16) -> u16 {
        let lo = self.read(addr);
        let hi = self.read(addr + 1);
        ((hi as u16) << 8) | lo as u16
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0..=0x7fff => self.cartridge.write(addr, value),
            0x8000..=0x9fff => self.io.write(addr, value),
            0xa000..=0xbfff => self.cartridge.write(addr, value),
            0xc000..=0xdfff => self.ram.write(addr, value),
            0xe000..=0xfdff => self.ram.write(addr - 0x2000, value),
            0xfe00..=0xfe9f => self.io.write(addr, value),
            0xff0f => INTERRUPT_FLAGS.set(value),
            0xff46 => self.dma_start(value),
            0xff50 => self.booting = false,
            0xff00..=0xff7f => self.io.write(addr, value),
            0xff80..=0xfffe => self.ram.write(addr, value),
            0xffff => INTERRUPT_ENABLE.set(value),
            _ => warn!("bus.write: unhandled address 0x{addr:04X}"),
        }
    }

    fn dma_start(&mut self, addr_byte: u8) {
        if self.dma.is_some() {
            error!("Overwriting DMA!");
        }

        self.dma = Some(DMA {
            src: addr_byte as u16 * 0x100,
            dest: 0xfe00,
        });
    }

    pub fn dma_tick(&mut self) {
        if self.dma.is_none() {
            return;
        }

        let mut dma = self.dma.as_mut().unwrap().clone();

        let value = self.read(dma.src);
        self.write(dma.dest, value);
        dma.src += 1;
        dma.dest += 1;

        if dma.dest > 0xfe9f {
            self.dma = None;
            return;
        }

        self.dma = Some(dma);
    }
}
