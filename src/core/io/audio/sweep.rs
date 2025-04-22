#[derive(Default)]
pub struct Sweep {
    pace: u8,
    direction: u8,
    step: u8,

    shadow: u16,
    enabled: bool,

    count: u8,
}

#[derive(Debug)]
pub struct FreqOverflow;

impl Sweep {
    pub fn set(&mut self, value: u8) {
        self.step = value & 0b111;
        self.direction = (value >> 3) & 1;
        self.pace = (value >> 4) & 0b111;
    }

    pub fn get(&self) -> u8 {
        (self.pace << 4) | (self.direction << 3) | self.step
    }

    pub fn tick(&mut self) -> Result<Option<u16>, FreqOverflow> {
        if !self.enabled() || self.pace == 0 {
            return Ok(None);
        }

        self.count += 1;

        if self.count == self.pace {
            self.count = 0;

            let new = self.check_freq_overflow()?;
            self.shadow = new;
            return Ok(Some(new));
        }
        Ok(None)
    }

    pub fn check_freq_overflow(&self) -> Result<u16, FreqOverflow> {
        let new = self.sweep();

        if new > 0x7ff {
            return Err(FreqOverflow);
        }
        Ok(new)
    }

    pub fn sweep(&self) -> u16 {
        let delta = self.shadow >> self.step;
        if self.direction == 1 {
            self.shadow + delta
        } else {
            self.shadow.saturating_sub(delta)
        }
    }

    pub fn trigger(&mut self, period: u16) -> Result<(), FreqOverflow> {
        self.count = 0;
        self.shadow = period;
        self.enabled = self.step != 0 || self.pace != 0;

        if self.step > 0 {
            self.check_freq_overflow()?;
        }
        Ok(())
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }
}
