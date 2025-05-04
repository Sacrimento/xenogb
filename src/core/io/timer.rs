use crate::core::cpu::interrupts::{request_interrupt, InterruptFlags};

pub struct Timer {
    div: u16,
    tima: u8,
    tma: u8,
    tac: u8,

    prev_div_bit: bool,
}

#[allow(clippy::new_without_default)]
impl Timer {
    pub fn new() -> Self {
        Self {
            div: 0,
            tima: 0,
            tma: 0,
            tac: 0,
            prev_div_bit: false,
        }
    }

    pub fn tick(&mut self) -> bool {
        let div_apu_bit = self.div_apu_bit();
        self.div = self.div.wrapping_add(4);
        let div_apu = div_apu_bit && !self.div_apu_bit();

        let current_bit = self.div_bit();

        if self.timer_enabled() && self.prev_div_bit && !current_bit {
            self.inc_tima();
        }

        self.prev_div_bit = current_bit;

        div_apu
    }

    fn div_bit(&self) -> bool {
        let bit_idx = match self.tac & 0b11 {
            0b00 => 9,
            0b01 => 3,
            0b10 => 5,
            0b11 => 7,
            _ => unreachable!(),
        };
        (self.div >> bit_idx) & 1 == 1
    }

    #[inline]
    fn div_apu_bit(&self) -> bool {
        (self.div >> 12) & 1 == 1
    }

    fn inc_tima(&mut self) {
        self.tima = self.tima.wrapping_add(1);

        if self.tima == 0 {
            self.tima = self.tma;
            request_interrupt(InterruptFlags::TIMER);
        }
    }

    fn timer_enabled(&self) -> bool {
        self.tac & 0b100 != 0
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xff04 => (self.div >> 8) as u8,
            0xff05 => self.tima,
            0xff06 => self.tma,
            0xff07 => self.tac | 0xf8,
            _ => unreachable!(),
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0xff04 => {
                if self.timer_enabled() && self.div_bit() {
                    self.inc_tima();
                }

                self.div = 0;
                self.prev_div_bit = false;
            }
            0xff05 => self.tima = value,
            0xff06 => self.tma = value,
            0xff07 => {
                let old_bit = self.timer_enabled() && self.div_bit();
                self.tac = value & 0b111;
                let new_bit = self.timer_enabled() && self.div_bit();

                if old_bit && !new_bit {
                    self.inc_tima();
                }

                self.prev_div_bit = new_bit;
            }
            _ => unreachable!(),
        };
    }
}
