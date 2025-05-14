use log::warn;

use crate::core::io::audio::{
    envelope::Envelope,
    length_counter::LengthCounter,
    sweep::{FreqOverflow, Sweep},
};

const DUTY_TABLE: [[u8; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0], // 12.5%
    [0, 1, 1, 0, 0, 0, 0, 0], // 25%
    [0, 1, 1, 1, 1, 0, 0, 0], // 50%
    [1, 0, 0, 1, 1, 1, 1, 1], // 75%
];

#[derive(Default)]
pub struct PulseChannel {
    enabled: bool,
    div: u16,

    sweep: Option<Sweep>,
    pub envelope: Envelope,
    length_counter: LengthCounter,

    duty_idx: u8,
    wave_duty: u8,
    pub period: u16,
}

impl PulseChannel {
    pub fn new(sweep: bool) -> Self {
        let sweep = if sweep { Some(Sweep::default()) } else { None };

        Self {
            sweep,
            envelope: Envelope::default(),
            length_counter: LengthCounter::new(64),
            ..Default::default()
        }
    }

    pub fn tick(&mut self) {
        self.div += 1;

        if self.div >= 0x800 {
            self.div = self.period;
            self.duty_idx = (self.duty_idx + 1) % 8;
        }
    }

    pub fn freq_sweep(&mut self) {
        if self.sweep.as_ref().is_some_and(|s| s.enabled()) {
            match self.sweep.as_mut().unwrap().tick() {
                Ok(None) => (),
                Ok(Some(p)) => self.period = p,
                Err(FreqOverflow) => self.enabled = false,
            }
        }
    }

    pub fn tick_length_timer(&mut self) {
        if self.length_counter.tick() {
            self.enabled = false;
        }
    }

    pub fn reset(&mut self) {
        self.enabled = false;
        self.div = 0;

        self.duty_idx = 0;
        self.wave_duty = 0;
        self.period = 0;

        self.envelope = Envelope::default();
        self.length_counter.reset();

        if let Some(sweep) = &mut self.sweep {
            *sweep = Sweep::default();
        }
    }

    pub fn sample(&self) -> f32 {
        if !self.enabled || self.period < 8 {
            return 0.0;
        }

        let signal = DUTY_TABLE[self.wave_duty as usize][self.duty_idx as usize];
        let volume = self.envelope.volume();

        let digital = signal * volume;

        let analogic = (digital as f32 / 16.0) * 2.0 - 1.0;
        analogic
    }

    fn trigger(&mut self, div_apu: u8) {
        self.envelope.trigger();
        self.enabled = self.envelope.dac_enabled();

        if self.length_counter.trigger() && div_apu % 2 == 0 && self.length_counter.tick() {
            self.enabled = false;
        }

        self.div = self.period;
        self.duty_idx = 0;

        if let Some(sweep) = &mut self.sweep {
            if sweep.trigger(self.period).is_err() {
                self.enabled = false;
            }

            // // Perform overflow check a second time
            // if sweep.check_freq_overflow().is_err() {
            //     self.enabled = false;
            // }
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn write(&mut self, addr: u16, value: u8, div_apu: u8) {
        match addr {
            0xff10 => self.sweep.as_mut().unwrap().set(value),
            0xff11 | 0xff16 => {
                self.length_counter.set(value & 0x3f);
                self.wave_duty = (value >> 6) & 0x3;
            }
            0xff12 | 0xff17 => {
                let prev_dac_enabled = self.envelope.dac_enabled();
                self.envelope.set(value);
                if prev_dac_enabled && !self.envelope.dac_enabled() {
                    self.enabled = false;
                }
            }
            0xff13 | 0xff18 => self.period = (self.period & 0x700) | value as u16,
            0xff14 | 0xff19 => {
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
            _ => unreachable!(),
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xff10 => self.sweep.as_ref().unwrap().get() | 0x80,
            0xff11 | 0xff16 => (self.wave_duty << 6) | 0x3f,
            0xff12 | 0xff17 => self.envelope.get(),
            0xff13 | 0xff18 => {
                warn!("PulseChannel: tried to read a write only value at 0x{addr:04X}");
                0xff
            }
            0xff14 | 0xff19 => ((self.length_counter.enabled() as u8) << 6) | 0xbf,
            _ => unreachable!(),
        }
    }
}
