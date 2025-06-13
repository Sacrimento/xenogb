use std::time::{Duration, Instant};

use log::{info, warn};

use crate::core::io::video::ppu::TICKS_PER_FRAME;

pub const CLOCK_SPEED: u32 = 4194304;
pub const DOUBLE_CLOCK_SPEED: u32 = CLOCK_SPEED * 2;

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy)]
pub enum CPUSpeed {
    NORMAL,
    DOUBLE,
    CUSTOM,
}

// CPU Clock based on ticks per frames instead of tick duration
pub struct Clock {
    frame_start: Instant,
    frame_target_duration: Duration,

    pub speed_mode: CPUSpeed,
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
            speed_mode: CPUSpeed::NORMAL,
            clock_ticks: 0,
        }
    }

    pub fn tick(&mut self) {
        self.clock_ticks += 4;

        if self.clock_ticks >= TICKS_PER_FRAME {
            self.clock_ticks = 0;
            let elapsed = self.frame_start.elapsed();
            if self.frame_target_duration > elapsed {
                std::thread::sleep(self.frame_target_duration - elapsed);
            } else if self.frame_target_duration > Duration::ZERO {
                let r = elapsed - self.frame_target_duration;
                if r > Duration::from_millis(1) {
                    warn!("CPU is behind by {:?}!", r);
                }
            }
            self.frame_start = Instant::now();
        }
    }

    pub fn switch_speed(&mut self, double_speed: bool) {
        match double_speed {
            false => {
                self.speed_mode = CPUSpeed::NORMAL;
                self.frame_target_duration =
                    Duration::from_secs_f64(TICKS_PER_FRAME as f64 / CLOCK_SPEED as f64);
            }
            true => {
                self.speed_mode = CPUSpeed::DOUBLE;
                self.frame_target_duration =
                    Duration::from_secs_f64(TICKS_PER_FRAME as f64 / DOUBLE_CLOCK_SPEED as f64);
            }
        }
        info!("Speed switched to {:?}", self.speed_mode);
    }

    pub fn set_speed(&mut self, clock_speed: u32) {
        self.frame_target_duration =
            Duration::from_secs_f64(TICKS_PER_FRAME as f64 / clock_speed as f64);
        self.speed_mode = CPUSpeed::CUSTOM;
    }
}
