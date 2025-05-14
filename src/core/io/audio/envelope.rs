#[derive(Default)]
pub struct Envelope {
    init_vol: u8,
    direction: u8,
    period: u8,

    volume: u8,

    timer: u8,
}

impl Envelope {
    pub fn set(&mut self, value: u8) {
        self.period = value & 0x7;
        self.direction = (value >> 3) & 0x1;
        self.init_vol = (value >> 4) & 0xf;
        self.volume = self.init_vol;
    }

    pub fn get(&self) -> u8 {
        (self.init_vol << 4) | (self.direction << 3) | self.period
    }

    pub fn tick(&mut self) {
        // if !self.dac_enabled() || self.period == 0 {
        //     return;
        // }

        if self.period == 0 {
            return;
        }

        if self.timer > 0 {
            self.timer -= 1;
        }

        if self.timer == 0 {
            self.timer = self.period;

            if self.direction == 1 && self.volume < 15 {
                self.volume += 1;
            } else if self.direction == 0 && self.volume > 0 {
                self.volume -= 1;
            }
        }
    }

    pub fn volume(&self) -> u8 {
        self.volume
    }

    pub fn trigger(&mut self) {
        self.volume = self.init_vol;
        self.timer = self.period;
    }

    pub fn dac_enabled(&self) -> bool {
        ((self.init_vol << 4) | (self.direction << 3)) & 0xf8 > 0
    }
}
