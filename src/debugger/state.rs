use std::time::Instant;

use super::disas::{disas, GbAsm};
use super::metrics::{CpuMetrics, MetricsExport, PpuMetrics};
use super::{CPU_METRICS, PPU_METRICS};
use crate::core::cpu::cpu::{CPURegisters, LR35902CPU};
use crate::core::cpu::interrupts::{INTERRUPT_ENABLE, INTERRUPT_FLAGS};
use crate::core::io::audio::apu::APU;
use crate::core::io::video::ppu::PPU;
use crate::core::run_emu::EmuCrash;

#[derive(Default)]
pub struct EmuSnapshot {
    pub ppu: PpuState,
    pub cpu: CpuState,
    pub breakpoints: Vec<u16>,
    pub apu: ApuState,
    pub crash: Option<EmuCrash>,
}

#[derive(Default)]
pub struct InterruptState {
    pub int_master: bool,
    pub int_enable: u8,
    pub interrupts: u8,
}

#[derive(Default)]
pub struct CpuState {
    pub registers: CPURegisters,
    pub halt: bool,
    pub interrupts: InterruptState,
    pub metrics: MetricsExport<CpuMetrics>,
    pub disas: Vec<GbAsm>,
}

impl CpuState {
    pub fn new(cpu: &LR35902CPU, last_pc: u16) -> Self {
        Self {
            registers: cpu.registers,
            halt: cpu.halt,
            interrupts: InterruptState {
                int_master: cpu.int_master,
                int_enable: INTERRUPT_ENABLE.get(),
                interrupts: INTERRUPT_FLAGS.get(),
            },
            metrics: CPU_METRICS.with_borrow(|mh| mh.export()),
            disas: disas(cpu, last_pc, 30),
        }
    }
}

#[derive(Default)]
pub struct ChannelState {
    pub enabled: bool,
    pub muted: bool,
    pub volume: u8,
    pub freq: u32,
}

pub struct Sample {
    pub sample: f32,
    pub at: Instant,
}

impl Default for Sample {
    fn default() -> Self {
        Self {
            sample: 0.0,
            at: Instant::now(),
        }
    }
}

#[derive(Default)]
pub struct ApuState {
    pub channel1: ChannelState,
    pub channel2: ChannelState,
    pub channel3: ChannelState,
    pub channel4: ChannelState,
    pub sample: Sample,
}

impl ApuState {
    pub fn new(apu: &APU) -> Self {
        let ch3_vol = [0, 15, 11, 4][apu.channel3.volume as usize];

        Self {
            channel1: ChannelState {
                enabled: apu.channel1.enabled(),
                muted: apu.channel1.muted,
                volume: apu.channel1.envelope.volume(),
                freq: 131072u32 / (2048u32 - apu.channel1.period as u32),
            },
            channel2: ChannelState {
                enabled: apu.channel2.enabled(),
                muted: apu.channel2.muted,
                volume: apu.channel2.envelope.volume(),
                freq: 131072u32 / (2048u32 - apu.channel2.period as u32),
            },
            channel3: ChannelState {
                enabled: apu.channel3.enabled(),
                muted: apu.channel3.muted,
                volume: ch3_vol,
                freq: 65536u32 / (2048u32 - apu.channel3.period as u32),
            },
            channel4: ChannelState {
                enabled: apu.channel4.enabled(),
                muted: apu.channel4.muted,
                volume: apu.channel4.envelope.volume(),
                freq: 0,
            },
            sample: Sample {
                sample: apu.last_sample,
                at: apu.last_sample_at,
            },
        }
    }
}

pub struct PpuState {
    pub vram: [u8; 0x2000],
    pub metrics: MetricsExport<PpuMetrics>,
}

impl Default for PpuState {
    fn default() -> Self {
        Self {
            vram: [0; 0x2000],
            metrics: MetricsExport::default(),
        }
    }
}

impl PpuState {
    pub fn new(ppu: &PPU) -> Self {
        Self {
            vram: ppu.vram,
            metrics: PPU_METRICS.with_borrow(|mh| mh.export()),
        }
    }
}
