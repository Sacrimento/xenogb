use chrono::{DateTime, Duration, TimeDelta, Utc};
use log::warn;
use std::path::PathBuf;

use super::MemoryBankController;

#[derive(Default, Debug)]
pub struct RTC {
    start: DateTime<Utc>,
    latch_reg: u8,

    seconds: u8,
    minutes: u8,
    hours: u8,
    days: u16,
}

impl RTC {
    pub fn new() -> Self {
        Self {
            start: Utc::now(),
            ..Default::default()
        }
    }

    pub fn latch(&mut self) {
        let now = Utc::now();
        let mut elapsed = now - self.start;

        self.days = elapsed.num_days() as u16;
        elapsed -= Duration::days(elapsed.num_days());
        self.hours = elapsed.num_hours() as u8;
        elapsed -= Duration::hours(elapsed.num_hours());
        self.minutes = elapsed.num_minutes() as u8;
        elapsed -= Duration::minutes(elapsed.num_minutes());
        self.seconds = elapsed.num_seconds() as u8;
    }

    pub fn read(&self, register: usize) -> u8 {
        match register {
            0x8 => self.seconds,
            0x9 => self.minutes,
            0xa => self.hours,
            0xb => (self.days & 0xff) as u8,
            0xc => ((self.days >> 8) & 1) as u8,
            _ => unreachable!(),
        }
    }

    pub fn write(&mut self, register: usize, value: u8) {
        self.latch();

        match register {
            0x8 => self.seconds = value,
            0x9 => self.minutes = value,
            0xa => self.hours = value,
            0xb => self.days = (self.days & 0x0f00) | value as u16,
            0xc => self.days = (self.days & 0xff) | ((value as u16 & 1) << 8),
            _ => unreachable!(),
        }

        let td = TimeDelta::days(self.days as i64)
            + TimeDelta::hours(self.hours as i64)
            + TimeDelta::minutes(self.minutes as i64)
            + TimeDelta::seconds(self.seconds as i64);
        self.start = Utc::now() - td;
    }
}

pub struct MBC3 {
    rom: Vec<u8>,
    ram_rtc_enable: bool,

    rom_bank: usize,
    ram_bank_rtc_reg: usize,

    sram: Vec<[u8; 0x2000]>,

    save_fname: PathBuf,

    has_save: bool,
    rtc: Option<RTC>,
}

impl MBC3 {
    pub fn new(
        rom: Vec<u8>,
        ram_bank_code: u8,
        has_save: bool,
        has_rtc: bool,
        mut rom_fname: PathBuf,
    ) -> Self {
        rom_fname.set_extension("gbsave");

        Self {
            rom,
            ram_rtc_enable: false,
            rom_bank: 1,
            ram_bank_rtc_reg: 0,
            sram: Self::load_sram(&rom_fname, has_save, ram_bank_code),
            save_fname: rom_fname,
            has_save,
            rtc: if has_rtc { Some(RTC::new()) } else { None },
        }
    }
}

impl MemoryBankController for MBC3 {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0..=0x3fff => self.rom[addr as usize],
            0x4000..=0x7fff => self.rom[self.rom_bank * 0x4000 + (addr as usize - 0x4000)],
            0xa000..=0xbfff => match self.ram_bank_rtc_reg {
                0x0..=0x7 => self.sram[self.ram_bank_rtc_reg][addr as usize - 0xa000],
                0x8..=0xc => {
                    if !self.ram_rtc_enable {
                        return 0xff;
                    }
                    if let Some(rtc) = &self.rtc {
                        rtc.read(self.ram_bank_rtc_reg)
                    } else {
                        0xff
                    }
                }
                _ => unreachable!(),
            },
            _ => {
                warn!("mbc3.read: unhandled address 0x{addr:04X}");
                0xff
            }
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0..=0x1fff => self.ram_rtc_enable = value == 0x0a,
            0x2000..=0x3fff => self.rom_bank = (value as usize & 0x7f).max(1),
            0x4000..=0x5fff => self.ram_bank_rtc_reg = value as usize & 0xf,
            0x6000..=0x7fff => {
                if let Some(rtc) = &mut self.rtc {
                    if rtc.latch_reg == 0 && value == 1 {
                        rtc.latch();
                    }
                    rtc.latch_reg = value;
                }
            }
            0xa000..=0xbfff => {
                if self.ram_rtc_enable {
                    match self.ram_bank_rtc_reg {
                        0x0..=0x7 => {
                            self.sram[self.ram_bank_rtc_reg][addr as usize - 0xa000] = value;
                        }
                        0x8..=0xc => {
                            if self.ram_rtc_enable {
                                if let Some(rtc) = &mut self.rtc {
                                    rtc.write(self.ram_bank_rtc_reg, value);
                                }
                            }
                        }
                        _ => unreachable!(),
                    }
                }
            }
            _ => warn!("mbc3.write: unhandled address 0x{addr:04X}"),
        }
    }

    fn save(&self) {
        if self.has_save {
            Self::save_sram(&self.save_fname, &self.sram);
        }
    }
}
