pub mod audio;
pub mod joypad;
mod serial;
mod timer;
pub mod video;
use crossbeam_channel::Sender;

use audio::apu::APU;
use joypad::Joypad;
use log::{info, warn};
use serial::Serial;
use timer::Timer;
use video::ppu::{Vbuf, PPU};

pub struct IOMMU {
    pub serial: Serial,
    pub timer: Timer,
    pub ppu: PPU,
    pub apu: APU,
    pub joypad: Joypad,
}

impl IOMMU {
    pub fn new(video_channel_sd: Sender<Vbuf>, audio_channel_sd: Sender<f32>) -> Self {
        Self {
            serial: Serial::default(),
            timer: Timer::new(),
            ppu: PPU::new(video_channel_sd),
            apu: APU::new(audio_channel_sd),
            joypad: Joypad::new(),
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x8000..=0x9fff => self.ppu.write(addr, value),
            0xfe00..=0xfe9f => self.ppu.write(addr, value),
            0xff00 => self.joypad.write(value),
            0xff01..=0xff02 => self.serial.write(addr, value),
            0xff04..=0xff07 => self.timer.write(addr, value),
            0xff10..=0xff26 => self.apu.write(addr, value),
            0xff30..=0xff3f => self.apu.write(addr, value),
            0xff40..=0xff4b => self.ppu.lcd.write(addr, value),
            0xff4f => self.ppu.write(addr, value),
            0xff68..=0xff6b => self.ppu.lcd.write(addr, value),
            _ => warn!("iommu.write: unhandled address 0x{addr:04X}"),
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0x9fff => self.ppu.read(addr),
            0xfe00..=0xfe9f => self.ppu.read(addr),
            0xff00 => self.joypad.read(),
            0xff01..=0xff02 => self.serial.read(addr),
            0xff04..=0xff07 => self.timer.read(addr),
            0xff10..=0xff26 => self.apu.read(addr),
            0xff30..=0xff3f => self.apu.read(addr),
            0xff40..=0xff4b => self.ppu.lcd.read(addr),
            0xff4f => self.ppu.read(addr),
            0xff68..=0xff6b => self.ppu.lcd.read(addr),
            _ => {
                warn!("iommu.read: unhandled address 0x{addr:04X}");
                0xff
            }
        }
    }
}
