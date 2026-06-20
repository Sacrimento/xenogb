use crate::core::cpu::interrupts::{request_interrupt, InterruptFlags};
use crate::core::cpu::CPUSpeed;
use crate::flag_set;

pub struct Timer {
    div: u16,
    tima: u8,
    tma: u8,
    tac: u8,

    prev_div_bit: bool,

    // Overflow delay state machine (matches潘 docs 2-cycle behavior):
    // Cycle A: tima overflows FF->00, overflow_delay = true
    // Cycle B: tick finalizes: tima = tma, IF set, was_reset = true
    // Cycle C: tick clears was_reset, timer resumes normal
    overflow_delay: bool,
    was_reset: bool,
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
            overflow_delay: false,
            was_reset: false,
        }
    }

    pub fn tick(&mut self, speed_mode: CPUSpeed) -> bool {
        let div_apu_bit = self.div_apu_bit(speed_mode);
        self.div = self.div.wrapping_add(4);
        let div_apu = div_apu_bit && !self.div_apu_bit(speed_mode);

        let current_bit = self.div_bit();

        if self.timer_enabled() {
            // Finalize overflow: reload TIMA from TMA and fire interrupt
            if self.overflow_delay {
                self.overflow_delay = false;
                self.tima = self.tma;
                request_interrupt(InterruptFlags::TIMER);
                self.was_reset = true;
            } else if self.was_reset {
                // Cycle after reload: clear was_reset, skip falling edge this cycle
                self.was_reset = false;
            } else {
                // Normal falling edge detection
                if self.prev_div_bit && !current_bit {
                    self.inc_tima();
                }
            }
        }

        self.prev_div_bit = current_bit;

        div_apu
    }

    #[inline(always)]
    fn div_bit(&self) -> bool {
        let bit_idx = match self.tac & 0b11 {
            0b00 => 9,
            0b01 => 3,
            0b10 => 5,
            0b11 => 7,
            _ => unreachable!(),
        };
        flag_set!(self.div >> bit_idx, 1)
    }

    #[inline(always)]
    fn div_apu_bit(&self, speed: CPUSpeed) -> bool {
        let bit = match speed {
            CPUSpeed::DOUBLE => 13,
            _ => 12,
        };
        flag_set!(self.div >> bit, 1)
    }

    fn inc_tima(&mut self) {
        self.tima = self.tima.wrapping_add(1);

        if self.tima == 0 {
            // Overflow: TIMA stays 0, reload deferred to next M-cycle
            self.overflow_delay = true;
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
            0xff05 => {
                // Writing to TIMA during overflow delay cycle cancels the pending reload
                if self.overflow_delay {
                    self.overflow_delay = false;
                }
                self.tima = value;
            }
            0xff06 => {
                self.tma = value;
                // Writing to TMA during was_reset cycle also updates TIMA
                if self.was_reset {
                    self.tima = value;
                    self.was_reset = false;
                }
            }
            0xff07 => {
                let was_enabled = self.timer_enabled();
                let old_bit = self.div_bit();
                self.tac = value & 0b111;
                let is_enabled = self.timer_enabled();
                let new_bit = self.div_bit();

                // Normal falling edge: enabled, bit goes 1 -> 0
                let falling_edge = was_enabled && old_bit && (!is_enabled || !new_bit);

                // DMG-specific: disabling timer with selected bit set triggers a tick
                let dmg_disable_tick = was_enabled && !is_enabled && old_bit;

                if falling_edge || dmg_disable_tick {
                    self.inc_tima();
                }

                self.prev_div_bit = is_enabled && new_bit;
            }
            _ => unreachable!(),
        };
    }
}
