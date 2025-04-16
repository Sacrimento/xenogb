#[derive(Default, Clone, Copy)]
pub struct LengthCounter {
    enabled: bool,
    pub value: u16,
    max: u16,
}

impl LengthCounter {
    pub fn new(max: u16) -> Self {
        Self {
            enabled: false,
            value: 0,
            max,
        }
    }

    pub fn set(&mut self, value: u8) {
        self.value = self.max - value as u16;
    }

    pub fn tick(&mut self) -> bool {
        // Returns wether the parent channel should be disabled
        if self.enabled && self.value > 0 {
            self.value -= 1;
            if self.value == 0 {
                return true;
            }
        }
        return false;
    }

    pub fn trigger(&mut self) -> bool {
        // Returns wether the length counter was "unfrozen"
        if self.value == 0 {
            self.value = self.max;
            return self.enabled;
        }
        false
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn reset(&mut self) {
        self.enabled = false;
        self.value = 0;
    }
}
