use log::{debug, error, info};

use crate::{flag_set, set_u16_hi, set_u16_lo};

const DMA_MODE: u8 = 0x80;

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug)]
pub enum VramDMAMode {
    IDLE,
    HBLANK,
    GENERAL,
}

pub struct VramDMA {
    pub src: u16,
    pub dst: u16,
    pub mode: VramDMAMode,
    pub remaining: u16,
}

impl std::fmt::Debug for VramDMA {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "VramDMA src=0x{:04X} dst=0x{:04X} mode={:#?} length=0x{:04X}",
            self.src, self.dst, self.mode, self.remaining
        )
    }
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
                self.remaining = (((value as u16) & 0b1111111) + 1) * 16;
                // Ignore 4 lower bits
                self.src &= 0xfff0;
                // Force address to be within VRAM & ignore 4 lower bits
                self.dst = (self.dst & 0xfff0) | 0x8000;
                self.mode = match flag_set!(value, DMA_MODE) {
                    true => VramDMAMode::HBLANK,
                    false => VramDMAMode::GENERAL,
                };
                debug!("{:?}", self);
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
                _ => ((self.remaining / 16) - 1) as u8,
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
