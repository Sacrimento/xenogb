use super::metrics::{CpuMetrics, MetricsExport};
use super::CPU_METRICS;
use crate::cpu::cpu::{CPURegisters, LR35902CPU};
use crate::cpu::interrupts::{INTERRUPT_ENABLE, INTERRUPT_FLAGS};
use crate::flag_set;
use crate::io::video::lcd::LCDC_FLAGS;
use std::time::Instant;

pub struct EmulationState {
    pub ppu: PpuState,
    pub cpu: CpuState,
}

pub struct InterruptState {
    pub int_master: bool,
    pub int_enable: u8,
    pub interrupts: u8,
}

pub struct CpuState {
    pub registers: CPURegisters,
    pub halt: bool,
    pub interrupts: InterruptState,
    pub metrics: MetricsExport<CpuMetrics>,
}

impl CpuState {
    pub fn new(cpu: &LR35902CPU) -> Self {
        Self {
            registers: cpu.registers.clone(),
            halt: cpu.halt,
            interrupts: InterruptState {
                int_master: cpu.int_master,
                int_enable: INTERRUPT_ENABLE.get(),
                interrupts: INTERRUPT_FLAGS.get(),
            },
            metrics: CPU_METRICS.with_borrow(|mh| mh.export()),
        }
    }
}

#[derive(Clone)]
pub struct PpuState {
    pub vram: [u8; 0x2000],
    pub frame_sprites: Vec<u8>,
    next_frame_sprites: Vec<u8>,

    frame: Instant,
}

impl PpuState {
    pub fn new() -> Self {
        Self {
            vram: [0; 0x2000],
            frame_sprites: vec![],
            next_frame_sprites: vec![],
            frame: Instant::now(),
        }
    }

    pub fn update(&mut self, cpu: &LR35902CPU) {
        if cpu.bus.io.ppu.last_frame != self.frame {
            self.frame = cpu.bus.io.ppu.last_frame;
            self.frame_sprites = self.next_frame_sprites.clone();
            self.next_frame_sprites.clear();
        }

        let mut sprite_idxs: Vec<u8> = vec![];
        if let Some(line_sprites) = cpu.bus.io.ppu.line_sprites.clone() {
            sprite_idxs = line_sprites
                .iter()
                .flat_map(|s| {
                    if flag_set!(cpu.bus.io.ppu.lcd.lcdc, LCDC_FLAGS::OBJ_SIZE) {
                        [s.tile_idx & 0xfe, s.tile_idx | 1]
                    } else {
                        [s.tile_idx, s.tile_idx]
                    }
                })
                .collect();
        }

        self.next_frame_sprites.extend(sprite_idxs);
        self.vram = cpu.bus.io.ppu.vram.clone();
    }
}
