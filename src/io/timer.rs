use crate::interrupts::{request_interrupt, InterruptFlags};

pub struct Timer {
    div: u16,
    tima: u8,
    tma: u8,
    tac: u8,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            div: 0xac00,
            tima: 0,
            tma: 0,
            tac: 0,
        }
    }

    pub fn tick(&mut self) {
        let prev_div = self.div;
        let mut update: bool = false;

        self.div += 1;

        match self.tac & 0b11 {
            0b00 => {
                update = (prev_div & (1 << 9) == 1) && (self.div & (1 << 9) == 0);
            }
            0b01 => {
                update = (prev_div & (1 << 3) == 1) && (self.div & (1 << 3) == 0);
            }
            0b10 => {
                update = (prev_div & (1 << 5) == 1) && (self.div & (1 << 5) == 0);
            }
            0b11 => {
                update = (prev_div & (1 << 7) == 1) && (self.div & (1 << 7) == 0);
            }
            _ => panic!("Unreachable"),
        }

        if update && (self.tac & (1 << 2) == 1) {
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
            _ => panic!("Unreachable"),
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0xff04 => self.div = 0,
            0xff05 => self.tima = value,
            0xff06 => self.tma = value,
            0xff07 => self.tac = value,
            _ => panic!("Unreachable"),
        };
    }
}
