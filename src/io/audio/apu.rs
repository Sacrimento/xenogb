use super::channel::PulseChannel;

#[allow(nonstandard_style)]
pub mod APU_AMC_FLAGS {
    pub const CH1_ON: u8 = 0x1;
    pub const CH2_ON: u8 = 0x2;
    pub const CH3_ON: u8 = 0x4;
    pub const CH4_ON: u8 = 0x8;
    pub const AUDIO_ON: u8 = 0x80;
}

#[allow(nonstandard_style)]
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

#[allow(nonstandard_style)]
pub mod APU_MVVP_FLAGS {
    pub const VOLUME_RIGHT: u8 = 0x7;
    pub const VIN_RIGHT: u8 = 0x8;
    pub const VOLUME_LEFT: u8 = 0x70;
    pub const VIN_LEFT: u8 = 0x80;
}

#[derive(Default)]
pub struct APU {
    master_control: u8,
    panning: u8,
    master_volume: u8,

    wave_ram: [u8; 0x10],
}

impl APU {
    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xff24 => self.master_volume,
            0xff25 => self.panning,
            0xff26 => self.master_control,
            0xff30..=0xff3f => self.wave_ram[addr as usize - 0xff30],
            _ => unreachable!(),
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0xff24 => self.master_volume = value,
            0xff25 => self.panning = value,
            0xff26 => self.master_control = value,
            0xff30..=0xff3f => self.wave_ram[addr as usize - 0xff30] = value,
            _ => unreachable!(),
        }
    }

    pub fn tick(&mut self, cycles: u8, apu_tick: bool) {
        if !apu_tick {
            return;
        }

        // self.channel1.freq_sweep();

        // self.channel1.tick();
        // self.channel2.tick();
        // self.channel3.tick();
        // self.channel4.tick();
    }
}
