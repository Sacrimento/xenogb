use crate::between;
mod serial;
mod timer;

use serial::Serial;
use timer::Timer;

pub struct IOMMU {
    pub serial: Serial,
    pub timer: Timer,
}

impl IOMMU {
    pub fn new() -> Self {
        Self {
            serial: Serial::default(),
            timer: Timer::new(),
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        if addr == 0xff00 {
            // Joypad input
            // println!("Unhandled io.write at 0x{:04X}", addr);
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
            // println!("Unhandled io.write at 0x{:04X}", addr);
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
            // BG / OBJ Palettes
            // println!("Unhandled io.write at 0x{:04X}", addr);
        } else if addr == 0xff70 {
            // WRAM Bank Select
            // println!("Unhandled io.write at 0x{:04X}", addr);
        } else {
            println!("Invalid addr 0x{:02x} for io.write", addr);
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        if addr == 0xff00 {
            // Joypad input
            println!("Unhandled io.read at 0x{:04X}", addr);
            0xff
        } else if between!(addr, 0xff01, 0xff02) {
            // Serial Transfer
            self.serial.read(addr)
        } else if between!(addr, 0xff04, 0xff07) {
            // Timer and divider
            self.timer.read(addr)
        } else if between!(addr, 0xff10, 0xff26) {
            // Audio
            println!("Unhandled io.read at 0x{:04X}", addr);
            0xff
        } else if between!(addr, 0xff30, 0xff3f) {
            // Wave pattern
            println!("Unhandled io.read at 0x{:04X}", addr);
            0xff
        } else if addr == 0xff44 {
            0x90
        } else if between!(addr, 0xff40, 0xff4b) {
            // LCD
            println!("Unhandled io.read at 0x{:04X}", addr);
            0xff
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
