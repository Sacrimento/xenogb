use crate::core::cpu::{
    interrupts::{request_interrupt, InterruptFlags},
    CLOCK_SPEED,
};

pub struct Timer {
    div: u16,
    tima: u8,
    tma: u8,
    tac: u8,

    ticks_since_inc: u32,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            div: 0xac00,
            tima: 0,
            tma: 0,
            tac: 0,
            ticks_since_inc: 0,
        }
    }

    pub fn tick(&mut self, cycles: u8) {
        self.div = self.div.wrapping_add(cycles as u16);

        if (self.tac & (1 << 2)) == 0 {
            return;
        }

        self.ticks_since_inc += cycles as u32 * 4;

        let ticks_per_inc: u32 = match self.tac & 0b11 {
            0b00 => CLOCK_SPEED / 4096,
            0b01 => CLOCK_SPEED / 262144,
            0b10 => CLOCK_SPEED / 65536,
            0b11 => CLOCK_SPEED / 16384,
            _ => unreachable!(),
        };

        if self.ticks_since_inc >= ticks_per_inc {
            self.ticks_since_inc %= ticks_per_inc;
            self.tima += 1;

            if self.tima == 0xff {
                self.tima = self.tma;

                request_interrupt(InterruptFlags::TIMER);
            }
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xff04 => (self.div >> 8) as u8,
            0xff05 => self.tima,
            0xff06 => self.tma,
            0xff07 => self.tac,
            _ => unreachable!(),
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0xff04 => self.div = 0,
            0xff05 => self.tima = value,
            0xff06 => self.tma = value,
            0xff07 => self.tac = value,
            _ => unreachable!(),
        };
    }
}
