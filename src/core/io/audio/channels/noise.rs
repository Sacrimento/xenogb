use crate::core::io::audio::{envelope::Envelope, length_counter::LengthCounter};
use log::warn;

#[derive(Default)]
pub struct NoiseChannel {
    enabled: bool,

    length_counter: LengthCounter,

    clock_div: u8,
    lfsr_width: bool,
    clock_shift: u8,

    lfsr: u16,
    div: u16,

    pub envelope: Envelope,
}

impl NoiseChannel {
    pub fn new() -> Self {
        Self {
            lfsr: 0x7fff,
            length_counter: LengthCounter::new(64),
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

        self.clock_div = 0;
        self.lfsr_width = false;
        self.clock_shift = 0;

        self.lfsr = 0;
        self.div = 0;

        self.envelope = Envelope::default();
        self.length_counter.reset();
    }

    pub fn tick(&mut self) {
        if !self.enabled {
            return;
        }

        if self.div > 0 {
            self.div -= 1;
        } else {
            self.div = self.period();

            let xor = (self.lfsr & 0x1) ^ (self.lfsr & 0x2);
            self.lfsr = (self.lfsr >> 1) | (xor << 14);

            if self.lfsr_width {
                self.lfsr = (self.lfsr & !(1 << 6)) | (xor << 6);
            }
        }
    }

    pub fn sample(&self) -> f32 {
        if !self.enabled || self.lfsr & 0x1 == 0 {
            return 0.0;
        }
        let digital = self.envelope.volume();
        let analogic = digital as f32 / 16.0 * 2.0 - 1.0;
        analogic
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    fn trigger(&mut self, div_apu: u8) {
        self.envelope.trigger();
        self.enabled = self.envelope.dac_enabled();

        self.lfsr = 0x7fff;

        if self.length_counter.trigger() && div_apu % 2 == 0 && self.length_counter.tick() {
            self.enabled = false;
        }

        self.div = self.period();
    }

    fn period(&self) -> u16 {
        let divisor = if self.clock_div == 0 {
            8
        } else {
            self.clock_div as u16 * 16
        };
        divisor << self.clock_shift
    }

    pub fn write(&mut self, addr: u16, value: u8, div_apu: u8) {
        match addr {
            0xff20 => self.length_counter.set(value & 0x3f),
            0xff21 => {
                let prev_dac_enabled = self.envelope.dac_enabled();
                self.envelope.set(value);
                if prev_dac_enabled && !self.envelope.dac_enabled() {
                    self.enabled = false;
                }
            }
            0xff22 => {
                self.clock_div = value & 0x7;
                self.lfsr_width = (value >> 3) & 0x1 == 1;
                self.clock_shift = (value >> 4) & 0xf;
            }
            0xff23 => {
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

                if (value >> 7) & 0x1 == 1 {
                    self.trigger(div_apu);
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xff20 => {
                warn!("NoiseChannel: tried to read a write only value at 0x{addr:04X}");
                0xff
            }
            0xff21 => self.envelope.get(),
            0xff22 => self.clock_div | ((self.lfsr_width as u8) << 3) | (self.clock_shift << 4),
            0xff23 => ((self.length_counter.enabled() as u8) << 6) | 0xbf,
            _ => unreachable!(),
        }
    }
}
