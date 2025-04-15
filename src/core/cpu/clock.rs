use std::time::{Duration, Instant};

use log::warn;

use crate::core::io::video::ppu::TICKS_PER_FRAME;

pub const CLOCK_SPEED: u32 = 4194304;

// CPU Clock based on ticks per frames instead of tick duration
pub struct Clock {
    frame_start: Instant,
    frame_target_duration: Duration,

    pub clock_ticks: u32,
}

impl Clock {
    pub fn new(clock_speed: u32) -> Self {
        let frame_target_duration = if clock_speed == u32::MAX {
            Duration::ZERO
        } else {
            Duration::from_secs_f64(TICKS_PER_FRAME as f64 / clock_speed as f64)
        };
        Self {
            frame_start: Instant::now(),
            frame_target_duration,
            clock_ticks: 0,
        }
    }

    pub fn tick(&mut self, clock_ticks: u32) {
        self.clock_ticks += clock_ticks;

        if self.clock_ticks >= TICKS_PER_FRAME {
            self.clock_ticks = 0;
            let elapsed = self.frame_start.elapsed();
            if self.frame_target_duration > elapsed {
                std::thread::sleep(self.frame_target_duration - elapsed);
            } else {
                let r = elapsed - self.frame_target_duration;
                if r > Duration::from_millis(1) {
                    warn!("CPU is behind by {:?}!", r);
                }
            }
            self.frame_start = Instant::now();
        }
    }

    pub fn set_speed(&mut self, clock_speed: u32) {
        self.frame_target_duration =
            Duration::from_secs_f64(TICKS_PER_FRAME as f64 / clock_speed as f64)
    }
}
