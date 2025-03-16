use crate::between;
use crate::cartridge::Cartridge;
use crate::interrupts::{INTERRUPT_ENABLE, INTERRUPT_FLAGS};
use crate::io::IOMMU;
use crate::ram::RAM;

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
    cartridge: Cartridge,
    ram: RAM,
    pub io: IOMMU,
}

impl Bus {
    pub fn new(cartridge: Cartridge) -> Self {
        Self {
            cartridge,
            ram: RAM::new(),
            io: IOMMU::new(),
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        if addr < 0x7fff {
            return self.cartridge.read(addr);
        } else if between!(addr, 0xc000, 0xdfff) {
            return self.ram.read(addr);
        } else if addr == 0xff0f {
            return INTERRUPT_FLAGS.get();
        } else if between!(addr, 0xff00, 0xff7f) {
            return self.io.read(addr);
        } else if between!(addr, 0xff80, 0xfffe) {
            return self.ram.read(addr);
        } else if addr == 0xffff {
            return INTERRUPT_ENABLE.get();
        }
        // println!("Unhandled bus.read at 0x{:04X}", addr);
        0xff
    }

    pub fn read16(&self, addr: u16) -> u16 {
        let lo = self.read(addr);
        let hi = self.read(addr + 1);
        ((hi as u16) << 8) | lo as u16
    }

    pub fn write(&mut self, addr: u16, value: u8) -> () {
        if addr < 0x7fff {
            todo!()
        } else if between!(addr, 0xc000, 0xdfff) {
            self.ram.write(addr, value);
        } else if addr == 0xff0f {
            INTERRUPT_FLAGS.set(value);
        } else if between!(addr, 0xff00, 0xff7f) {
            self.io.write(addr, value);
        } else if between!(addr, 0xff80, 0xfffe) {
            self.ram.write(addr, value);
        } else if addr == 0xffff {
            INTERRUPT_ENABLE.set(value);
        } else {
            // println!("Unhandled bus.write at 0x{:04X}", addr);
            // todo!();
        }
    }
}
