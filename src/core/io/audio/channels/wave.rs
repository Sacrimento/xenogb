use log::warn;

use crate::core::io::audio::length_counter::LengthCounter;

#[derive(Default)]
pub struct WaveChannel {
    enabled: bool,
    dac_enabled: bool,
    pub muted: bool,

    length_counter: LengthCounter,

    div: u16,

    pub volume: u8,
    pub period: u16,

    wave_ram_idx: usize,
    wave_ram: [u8; 0x10],
}

impl WaveChannel {
    pub fn new() -> Self {
        Self {
            wave_ram_idx: 1,
            length_counter: LengthCounter::new(256),
            ..Default::default()
        }
    }

    pub fn tick_length_timer(&mut self) {
        if self.length_counter.tick() {
            self.enabled = false;
        }
    }

    pub fn reset(&mut self) {
        self.enabled = false;
        self.dac_enabled = false;

        self.div = 0;

        self.volume = 0;
        self.period = 0;

        self.length_counter.reset();

        self.wave_ram_idx = 0;
    }

    pub fn sample(&self) -> f32 {
        if self.muted {
            return 0.0;
        }

        if !self.enabled || !self.dac_enabled {
            return 0.0;
        }

        let nibble = if self.wave_ram_idx % 2 == 0 { 4 } else { 0 };
        let sample = (self.wave_ram[self.wave_ram_idx / 2] >> nibble) & 0xf;

        if self.volume == 0 {
            return 0.0;
        }

        let digital = sample >> (self.volume - 1);
        digital as f32 / 15.0 * 2.0 - 1.0
    }

    pub fn tick(&mut self) {
        if !self.enabled || !self.dac_enabled {
            return;
        }

        self.div = self.div.saturating_sub(2);

        if self.div == 0 {
            self.div = (2048 - self.period) * 2;
            self.wave_ram_idx = (self.wave_ram_idx + 1) % 32;
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    fn trigger(&mut self, div_apu: u8) {
        self.enabled = self.dac_enabled;

        if self.length_counter.trigger() && div_apu % 2 == 0 && self.length_counter.tick() {
            self.enabled = false;
        }

        self.div = (2048 - self.period) * 2;
        self.wave_ram_idx = 1;
    }

    pub fn write(&mut self, addr: u16, value: u8, div_apu: u8) {
        match addr {
            0xff1a => {
                let prev_dac_enabled = self.dac_enabled;
                self.dac_enabled = (value >> 7) & 1 == 1;
                if prev_dac_enabled && !self.dac_enabled {
                    self.enabled = false;
                }
            }
            0xff1b => self.length_counter.set(value),
            0xff1c => self.volume = (value >> 5) & 0b11,
            0xff1d => self.period = (self.period & 0x700) | value as u16,
            0xff1e => {
                let lc_enabled = self.length_counter.enabled();

                self.length_counter.set_enabled(value & 0x40 == 0x40);
                if !lc_enabled
                    && self.length_counter.enabled()
                    && div_apu % 2 == 0
                    && self.length_counter.value > 0
                    && self.length_counter.tick()
                {
                    self.enabled = false;
                }

                self.period = (self.period & 0xff) | ((value as u16 & 0x7) << 8);

                if (value >> 7) & 1 == 1 {
                    self.trigger(div_apu);
                }
            }
            0xff30..=0xff3f => self.wave_ram[addr as usize - 0xff30] = value,
            _ => unreachable!(),
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xff1a => ((self.dac_enabled as u8) << 7) | 0x7f,
            0xff1b => {
                warn!("WaveChannel: tried to read a write only value at 0x{addr:04X}");
                0xff
            }
            0xff1c => (self.volume << 5) | 0x9f,
            0xff1d => {
                warn!("WaveChannel: tried to read a write only value at 0x{addr:04X}");
                0xff
            }
            0xff1e => ((self.length_counter.enabled() as u8) << 6) | 0xbf,
            0xff30..=0xff3f => {
                let nibble = if self.wave_ram_idx % 2 == 0 { 4 } else { 0 };
                (self.wave_ram[self.wave_ram_idx / 2] >> nibble) & 0xf
            }
            _ => unreachable!(),
        }
    }

    pub fn mute(&mut self) {
        self.muted = !self.muted;
    }
}
