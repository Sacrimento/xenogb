#[derive(Default)]
struct Sweep {
    pace: u8,
    direction: u8,
    step: u8,

    count: u8,
}

struct Envelope {
    init_vol: u8,
    direction: u8,
    sweep_pace: u8,
}

pub struct PulseChannel {
    enabled: bool,

    div: u16,

    sweep: Option<Sweep>,
    init_length_timer: u8,
    wave_duty: u8,
    envelope: Envelope,
    period: u16,
}

impl Sweep {
    pub fn tick(&mut self, period: u16) -> Option<u16> {
        if !self.enabled {
            return None;
        }

        self.count += 1;

        if self.count == self.pace {
            self.count = 0;
            let mut new = period / (2 * *self.step);
            if self.direction == 0 {
                new = -new;
            }
            return Some(period + new);
        }
        None
    }
}

impl PulseChannel {
    pub fn tick(&self) {
        if let Some(sweep) = &self.sweep {
            let period = sweep.tick(self.period);

            if period > 0x7ff {
                self.enabled = false;
            }
        }

        self.div += 1;

        if self.div == 0x800 {
            self.div = self.period;
        }
    }
}
