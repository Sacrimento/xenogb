use super::boot::{get_boot_rom, BootRom};
use super::cartridge::Cartridge;
use super::dma::{OamDMA, VramDMA, VramDMAMode};
use super::ram::RAM;
use crate::core::cpu::interrupts::{INTERRUPT_ENABLE, INTERRUPT_FLAGS};
use crate::core::io::video::{
    lcd::PPUMode,
    ppu::{Vbuf, RESX},
};
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

pub struct Bus {
    pub cartridge: Cartridge,
    ram: RAM,
    pub io: IOMMU,

    oam_dma: OamDMA,
    vram_dma: VramDMA,

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
            oam_dma: OamDMA::default(),
            vram_dma: VramDMA::default(),
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
            0xff46 => {
                warn!("Invalid DMA read at 0x{addr:04X}");
                0xff
            }
            0xff51..=0xff55 => self.vram_dma.read(addr),
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
            0xff46 => self.oam_dma.init(value),
            0xff50 => self.booting = false,
            0xff70 => self.ram.write(addr, value),
            0xff51..=0xff55 => self.vram_dma.write(addr, value),
            0xff00..=0xff7f => self.io.write(addr, value),
            0xff80..=0xfffe => self.ram.write(addr, value),
            0xffff => INTERRUPT_ENABLE.set(value),
            _ => warn!("bus.write: unhandled address 0x{addr:04X}"),
        }
    }

    pub fn tick(&mut self) {
        self.oam_dma_tick();
        self.vram_dma_tick();
    }

    fn oam_dma_tick(&mut self) {
        if self.oam_dma.src == 0 {
            return;
        }

        self.write(self.oam_dma.dst, self.read(self.oam_dma.src));
        self.oam_dma.src += 1;
        self.oam_dma.dst += 1;

        if self.oam_dma.dst > 0xfe9f {
            self.oam_dma.src = 0;
        }
    }

    fn vram_dma_tick(&mut self) {
        match self.vram_dma.mode {
            VramDMAMode::IDLE => (),
            VramDMAMode::HBLANK => {
                if self.io.ppu.lcd.get_ppu_mode() == PPUMode::HBlank {
                    if self.vram_dma.remaining == 0 {
                        self.vram_dma.mode = VramDMAMode::IDLE;
                        return;
                    }

                    // Start of HBlank, copy 16 bytes
                    if self.io.ppu.line_x as usize == RESX {
                        let src = self.vram_dma.src & 0b11111111_11110000;
                        let dst = self.vram_dma.dst & 0b00011111_11110000;

                        for i in 0..16 {
                            self.write(dst + i, self.read(src + i));
                        }
                        self.vram_dma.remaining -= 1;
                    }
                }
            }
            VramDMAMode::GENERAL => {
                let length = (self.vram_dma.remaining as u16 + 1) * 16;
                let src = self.vram_dma.src & 0b11111111_11110000;
                let dst = self.vram_dma.dst & 0b00011111_11110000;

                for i in 0..length {
                    self.write(dst + i, self.read(src + i));
                }

                self.vram_dma.remaining = 0;
                self.vram_dma.mode = VramDMAMode::IDLE;
            }
        }
    }
}
