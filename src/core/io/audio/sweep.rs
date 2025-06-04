#[derive(Default)]
pub struct Sweep {
    period: u8,
    direction: u8,
    shift: u8,

    shadow: u16,
    enabled: bool,

    timer: u8,
}

#[derive(Debug)]
pub struct FreqOverflow;

impl Sweep {
    pub fn set(&mut self, value: u8) {
        self.shift = value & 0b111;
        self.direction = (value >> 3) & 1;
        self.period = (value >> 4) & 0b111;
    }

    pub fn get(&self) -> u8 {
        (self.period << 4) | (self.direction << 3) | self.shift
    }

    // pub fn tick(&mut self) -> Result<Option<u16>, FreqOverflow> {
    //     if !self.enabled() || self.period == 0 {
    //         return Ok(None);
    //     }

    //     self.timer -= 1;

    //     if self.timer == 0 {
    //         self.timer = if self.period != 0 { self.period } else { 8 };

    //         let new = self.check_freq_overflow()?;
    //         self.shadow = new;
    //         return Ok(Some(new));
    //     }
    //     Ok(None)
    // }

    pub fn tick(&mut self) -> Result<Option<u16>, FreqOverflow> {
        if self.timer > 0 {
            self.timer -= 1;
        }

        if self.timer == 0 {
            self.timer = if self.period != 0 { self.period } else { 8 };

            if self.enabled {
                let new = self.check_freq_overflow()?;
                self.shadow = new;
                return Ok(Some(new));
            }
        }
        Ok(None)
    }

    pub fn check_freq_overflow(&mut self) -> Result<u16, FreqOverflow> {
        let new = self.sweep();

        if new > 0x7ff {
            self.enabled = false;
            return Err(FreqOverflow);
        }
        Ok(new)
    }

    pub fn sweep(&self) -> u16 {
        let delta = self.shadow >> self.shift;
        if self.direction == 1 {
            self.shadow + delta
        } else {
            self.shadow.saturating_sub(delta)
        }
    }

    pub fn trigger(&mut self, period: u16) -> Result<(), FreqOverflow> {
        self.timer = if self.period != 0 { self.period } else { 8 };
        self.shadow = period;
        self.enabled = self.shift != 0 || self.period != 0;

        if self.shift > 0 {
            self.check_freq_overflow()?;
        }
        Ok(())
    }
}
