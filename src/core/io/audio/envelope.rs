#[derive(Default)]
pub struct Envelope {
    init_vol: u8,
    direction: u8,
    sweep_pace: u8,

    volume: u8,

    counter: u8,
}

impl Envelope {
    pub fn set(&mut self, value: u8) {
        self.sweep_pace = value & 0x7;
        self.direction = (value >> 3) & 0x1;
        self.init_vol = (value >> 4) & 0xf;
        self.volume = self.init_vol;
    }

    pub fn get(&self) -> u8 {
        (self.init_vol << 4) | (self.direction << 3) | self.sweep_pace
    }

    pub fn tick(&mut self) {
        if !self.dac_enabled() || self.sweep_pace == 0 {
            return;
        }

        self.counter += 1;

        if self.counter == self.sweep_pace {
            self.counter = 0;
            // if self.volume == 15 || self.volume == 0 {
            //     self.enabled() = false;
            // } else {
            if self.direction == 1 && self.volume < 15 {
                self.volume += 1;
            } else if self.direction == 0 && self.volume > 0 {
                self.volume -= 1;
            }
            // }
        }
    }

    pub fn volume(&self) -> u8 {
        self.volume
    }

    pub fn trigger(&mut self) {
        self.volume = self.init_vol;
        self.counter = 0;
    }

    pub fn dac_enabled(&self) -> bool {
        ((self.init_vol << 4) | (self.direction << 3)) & 0xf8 > 0
    }
}
