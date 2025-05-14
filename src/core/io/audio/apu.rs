use crossbeam_channel::Sender;
use log::warn;

use super::channels::{NoiseChannel, PulseChannel, WaveChannel};
use crate::core::cpu::CLOCK_SPEED;
use crate::flag_set;

const SAMPLE_RATE: f32 = 44100.0;
const TICKS_PER_SAMPLE: f32 = CLOCK_SPEED as f32 / SAMPLE_RATE;

#[allow(nonstandard_style, dead_code)]
pub mod APU_AMC_FLAGS {
    pub const CH1_ON: u8 = 0x1;
    pub const CH2_ON: u8 = 0x2;
    pub const CH3_ON: u8 = 0x4;
    pub const CH4_ON: u8 = 0x8;
    pub const AUDIO_ON: u8 = 0x80;
}

#[allow(nonstandard_style, dead_code)]
pub mod APU_SP_FLAGS {
    pub const CH1_RIGHT: u8 = 0x1;
    pub const CH2_RIGHT: u8 = 0x2;
    pub const CH3_RIGHT: u8 = 0x4;
    pub const CH4_RIGHT: u8 = 0x8;
    pub const CH1_LEFT: u8 = 0x10;
    pub const CH2_LEFT: u8 = 0x20;
    pub const CH3_LEFT: u8 = 0x40;
    pub const CH4_LEFT: u8 = 0x80;
}

#[allow(nonstandard_style, dead_code)]
pub mod APU_MVVP_FLAGS {
    pub const VOLUME_RIGHT: u8 = 0x7;
    pub const VIN_RIGHT: u8 = 0x8;
    pub const VOLUME_LEFT: u8 = 0x70;
    pub const VIN_LEFT: u8 = 0x80;
}

pub struct APU {
    master_control: u8,
    panning: u8,
    master_volume: u8,

    div_apu: u8,

    pub channel1: PulseChannel,
    pub channel2: PulseChannel,
    pub channel3: WaveChannel,
    pub channel4: NoiseChannel,

    ticks_since_sample: f32,
    prev_sample: f32,
    audio_channel_sd: Sender<f32>,
}

impl APU {
    pub fn new(audio_channel_sd: Sender<f32>) -> Self {
        Self {
            master_control: 0,
            master_volume: 0,
            panning: 0,
            div_apu: 0,
            channel1: PulseChannel::new(true),
            channel2: PulseChannel::new(false),
            channel3: WaveChannel::new(),
            channel4: NoiseChannel::new(),
            ticks_since_sample: 0.0,
            prev_sample: 0.0,
            audio_channel_sd,
        }
    }

    fn reset(&mut self) {
        self.master_control = 0;
        self.panning = 0;
        self.master_volume = 0;

        self.channel1.reset();
        self.channel2.reset();
        self.channel3.reset();
        self.channel4.reset();
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xff10..=0xff14 => self.channel1.read(addr),
            0xff16..=0xff19 => self.channel2.read(addr),
            0xff1a..=0xff1e => self.channel3.read(addr),
            0xff20..=0xff23 => self.channel4.read(addr),
            0xff24 => self.master_volume,
            0xff25 => self.panning,
            0xff26 => {
                if self.enabled() {
                    0xf0 // APU_AMC_FLAGS::AUDIO_ON | 0x70
                        | (self.channel1.enabled() as u8)
                        | ((self.channel2.enabled() as u8) << 1)
                        | ((self.channel3.enabled() as u8) << 2)
                        | ((self.channel4.enabled() as u8) << 3)
                } else {
                    0x70
                }
            }
            0xff30..=0xff3f => self.channel3.read(addr),
            _ => {
                warn!("apu.read: unhandled address 0x{addr:04X}");
                0xff
            }
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        if !self.enabled() && !matches!(addr, 0xff20 | 0xff26 | 0xff30..=0xff3f) {
            return;
        }

        match addr {
            0xff10..=0xff14 => self.channel1.write(addr, value, self.div_apu),
            0xff16..=0xff19 => self.channel2.write(addr, value, self.div_apu),
            0xff1a..=0xff1e => self.channel3.write(addr, value, self.div_apu),
            0xff20..=0xff23 => self.channel4.write(addr, value, self.div_apu),
            0xff24 => self.master_volume = value,
            0xff25 => self.panning = value,
            0xff26 => {
                if !self.enabled() && flag_set!(value, APU_AMC_FLAGS::AUDIO_ON) {
                    self.master_control = APU_AMC_FLAGS::AUDIO_ON;
                } else if self.enabled() && !flag_set!(value, APU_AMC_FLAGS::AUDIO_ON) {
                    self.reset();
                }
            }
            0xff30..=0xff3f => self.channel3.write(addr, value, self.div_apu),
            _ => warn!("apu.write: unhandled address 0x{addr:04X}"),
        }
    }

    #[inline]
    fn enabled(&self) -> bool {
        flag_set!(self.master_control, APU_AMC_FLAGS::AUDIO_ON)
    }

    pub fn tick(&mut self, div_apu: bool) {
        if div_apu {
            self.div_apu = (self.div_apu + 1) % 8;

            if !self.enabled() {
                return;
            }

            if self.div_apu % 2 == 0 {
                self.channel1.tick_length_timer();
                self.channel2.tick_length_timer();
                self.channel3.tick_length_timer();
                self.channel4.tick_length_timer();
            }

            if self.div_apu % 4 == 2 {
                self.channel1.freq_sweep();
            }

            if self.div_apu == 7 {
                self.channel1.envelope.tick();
                self.channel2.envelope.tick();
                self.channel4.envelope.tick();
            }
        }

        if !self.enabled() {
            return;
        }
        self.ticks_since_sample += 4.0;

        self.channel1.tick();
        self.channel2.tick();
        self.channel3.tick();
        self.channel4.tick();

        if self.ticks_since_sample >= TICKS_PER_SAMPLE {
            self.ticks_since_sample -= TICKS_PER_SAMPLE;
            let s = self.mix();
            _ = self.audio_channel_sd.send(s);
        }
    }

    fn hpf(&mut self, sample: f32) {
        let alpha = 0.8;
        self.prev_sample = alpha * self.prev_sample + (1.0 - alpha) * sample;
    }

    fn mix(&mut self) -> f32 {
        let mut sample = 0.0;

        if self.channel1.enabled() {
            sample += self.channel1.sample();
        }
        if self.channel2.enabled() {
            sample += self.channel2.sample();
        }
        if self.channel3.enabled() {
            sample += self.channel3.sample();
        }
        if self.channel4.enabled() {
            sample += self.channel4.sample();
        }

        sample /= 4.0 as f32;
        // self.hpf(sample);
        // self.prev_sample
        sample
    }
}
