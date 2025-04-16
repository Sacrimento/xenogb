#[derive(Default)]
pub struct Sweep {
    pace: u8,
    direction: u8,
    step: u8,

    count: u8,
}

impl Sweep {
    pub fn set(&mut self, value: u8) {
        self.step = value & 0b111;
        self.direction = (value >> 3) & 1;
        self.pace = (value >> 4) & 0b111;
    }

    pub fn get(&self) -> u8 {
        (self.pace << 4) | (self.direction << 3) | self.step
    }

    pub fn tick(&mut self, period: u16) -> Option<u16> {
        if !self.enabled() || self.pace == 0 {
            return None;
        }

        self.count += 1;

        if self.count == self.pace {
            self.count = 0;
            return Some(self.sweep(period));
        }
        None
    }

    pub fn sweep(&mut self, period: u16) -> u16 {
        let delta = period >> self.step;
        let result = if self.direction == 1 {
            period + delta
        } else {
            period.saturating_sub(delta)
        };

        if result > 0x7ff {
            0x800
        } else {
            result
        }
    }

    pub fn trigger(&mut self) {
        self.count = 0;
    }

    pub fn enabled(&self) -> bool {
        self.step != 0 || self.pace != 0
    }
}
