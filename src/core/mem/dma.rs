use log::{error, info};

use crate::{flag_set, set_u16_hi, set_u16_lo};

const DMA_MODE: u8 = 0x80;

pub enum VramDMAMode {
    IDLE,
    HBLANK,
    GENERAL,
}

pub struct VramDMA {
    pub src: u16,
    pub dst: u16,
    pub mode: VramDMAMode,
    pub remaining: u8,
}

impl Default for VramDMA {
    fn default() -> Self {
        Self {
            src: 0xff,
            dst: 0xff,
            mode: VramDMAMode::IDLE,
            remaining: 0,
        }
    }
}

impl VramDMA {
    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0xff51 => self.src = set_u16_hi!(self.src, value),
            0xff52 => self.src = set_u16_lo!(self.src, value),
            0xff53 => self.dst = set_u16_hi!(self.dst, value),
            0xff54 => self.dst = set_u16_lo!(self.dst, value),
            0xff55 => {
                self.remaining = value & 0b1111111;
                self.mode = match flag_set!(value, DMA_MODE) {
                    true => VramDMAMode::HBLANK,
                    false => VramDMAMode::GENERAL,
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xff51..=0xff54 => {
                info!("Attempt to read write-only value @0x{addr:04X}");
                0xff
            }
            0xff55 => match self.remaining {
                0 => 0xff,
                _ => self.remaining,
            },
            _ => unreachable!(),
        }
    }
}

#[derive(Default)]
pub struct OamDMA {
    pub src: u16,
    pub dst: u16,
}

impl OamDMA {
    pub fn init(&mut self, src: u8) {
        if self.src > 0 {
            error!("Overwriting OAM DMA!");
        }

        self.src = src as u16 * 0x100;
        self.dst = 0xfe00;
    }
}
