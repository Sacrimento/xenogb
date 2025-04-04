use crate::between;
pub mod joypad;
mod serial;
mod timer;
pub mod video;

use joypad::Joypad;
use serial::Serial;
use timer::Timer;
use video::ppu::PPU;

pub struct IOMMU {
    pub serial: Serial,
    pub timer: Timer,
    pub ppu: PPU,
    pub joypad: Joypad,
}

impl IOMMU {
    pub fn new() -> Self {
        Self {
            serial: Serial::default(),
            timer: Timer::new(),
            ppu: PPU::new(),
            joypad: Joypad::new(),
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        if between!(addr, 0x8000, 0x9fff) {
            self.ppu.vram_write(addr, value);
        } else if between!(addr, 0xfe00, 0xfe9f) {
            self.ppu.oam_write(addr, value);
        } else if addr == 0xff00 {
            // Joypad input
            self.joypad.write(value);
        } else if between!(addr, 0xff01, 0xff02) {
            // Serial Transfer
            self.serial.write(addr, value);
        } else if between!(addr, 0xff04, 0xff07) {
            // Timer and divider
            self.timer.write(addr, value);
        } else if between!(addr, 0xff10, 0xff26) {
            // Audio
            // println!("Unhandled io.write at 0x{:04X}", addr);
        } else if between!(addr, 0xff30, 0xff3f) {
            // Wave pattern
            // println!("Unhandled io.write at 0x{:04X}", addr);
        } else if between!(addr, 0xff40, 0xff4b) {
            // LCD
            self.ppu.lcd.write(addr, value);
        } else if addr == 0xff4f {
            // VRAM Bank Select
            // println!("Unhandled io.write at 0x{:04X}", addr);
        } else if addr == 0xff50 {
            // Set to non-zero to disable boot ROM
            // println!("Unhandled io.write at 0x{:04X}", addr);
        } else if between!(addr, 0xff51, 0xff55) {
            // VRAM DMA
            // println!("Unhandled io.write at 0x{:04X}", addr);
        } else if between!(addr, 0xff68, 0xff6b) {
            // BG / OBJ Palettes (CGB ONLY)
        } else if addr == 0xff70 {
            // WRAM Bank Select
            // println!("Unhandled io.write at 0x{:04X}", addr);
        } else {
            // println!("Invalid addr 0x{:02x} for io.write", addr);
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        if between!(addr, 0x8000, 0x9fff) {
            self.ppu.vram_read(addr)
        } else if between!(addr, 0xfe00, 0xfe9f) {
            self.ppu.oam_read(addr)
        } else if addr == 0xff00 {
            // Joypad input
            self.joypad.read()
        } else if between!(addr, 0xff01, 0xff02) {
            // Serial Transfer
            self.serial.read(addr)
        } else if between!(addr, 0xff04, 0xff07) {
            // Timer and divider
            self.timer.read(addr)
        } else if between!(addr, 0xff10, 0xff26) {
            // Audio
            // println!("Unhandled io.read at 0x{:04X}", addr);
            0xff
        } else if between!(addr, 0xff30, 0xff3f) {
            // Wave pattern
            println!("Unhandled io.read at 0x{:04X}", addr);
            0xff
        } else if between!(addr, 0xff40, 0xff4b) {
            // LCD
            self.ppu.lcd.read(addr)
        } else if addr == 0xff4f {
            // VRAM Bank Select
            println!("Unhandled io.read at 0x{:04X}", addr);
            0xff
        } else if addr == 0xff50 {
            // Set to non-zero to disable boot ROM
            println!("Unhandled io.read at 0x{:04X}", addr);
            0xff
        } else if between!(addr, 0xff51, 0xff55) {
            // VRAM DMA
            println!("Unhandled io.read at 0x{:04X}", addr);
            0xff
        } else if between!(addr, 0xff68, 0xff6b) {
            // BG / OBJ Palettes
            println!("Unhandled io.read at 0x{:04X}", addr);
            0xff
        } else if addr == 0xff70 {
            // WRAM Bank Select
            println!("Unhandled io.read at 0x{:04X}", addr);
            0xff
        } else {
            0xff
            // panic!("Invalid addr 0x{:02x} for io.read", addr);
        }
    }
}
