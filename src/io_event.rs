use crate::cpu::cpu::LR35902CPU;
use crate::Playback;

use crossbeam_channel::Receiver;
use std::fmt::Display;

#[allow(nonstandard_style)]
#[derive(Debug, Clone, Copy)]
pub enum IOEvent {
    JOYPAD_PRESS(u8),
    JOYPAD_RELEASE(u8),
    CLOSE,
}

impl Display for IOEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            IOEvent::JOYPAD_PRESS(key) => write!(f, "PRESS {}", key),
            IOEvent::JOYPAD_RELEASE(key) => write!(f, "RELEASE {}", key),
            IOEvent::CLOSE => write!(f, "CLOSE"),
        }
    }
}

impl IOEvent {
    pub fn from_strs(s: &str, k: Option<&str>) -> Result<Self, ()> {
        match s {
            "PRESS" => {
                if let Some(k) = k {
                    return Ok(Self::JOYPAD_PRESS(
                        k.parse().expect("Could not parse press input"),
                    ));
                }
                Err(())
            }
            "RELEASE" => {
                if let Some(k) = k {
                    return Ok(Self::JOYPAD_RELEASE(
                        k.parse().expect("Could not parse release input"),
                    ));
                }
                Err(())
            }
            "CLOSE" => Ok(Self::CLOSE),
            _ => Err(()),
        }
    }
}

pub struct IOListener {
    event_rc: Receiver<IOEvent>,
}

impl IOListener {
    pub fn new(event_rc: Receiver<IOEvent>) -> Self {
        Self { event_rc }
    }

    pub fn handle_events(&self, cpu: &mut LR35902CPU, playback: &mut Playback) {
        let cpu_frames = cpu.bus.io.ppu.frames;
        let cpu_ticks = cpu.clock.clock_ticks;

        let mut dispatch_event = |event| match event {
            IOEvent::JOYPAD_PRESS(key) => cpu.bus.io.joypad.press(key),
            IOEvent::JOYPAD_RELEASE(key) => cpu.bus.io.joypad.release(key),
            IOEvent::CLOSE => cpu.bus.cartridge.mbc.save(),
        };

        if playback.player.enabled() {
            for event in playback.player.events(cpu_frames, cpu_ticks) {
                if playback.recorder.enabled() {
                    playback.recorder.record(&event, cpu_frames, cpu_ticks);
                }

                dispatch_event(event);
            }
            // Ignore inputs while we are replaying the run
            return;
        }

        if let Ok(event) = self.event_rc.try_recv() {
            if playback.recorder.enabled() {
                playback.recorder.record(&event, cpu_frames, cpu_ticks);
            }
            dispatch_event(event);
        }
    }
}
