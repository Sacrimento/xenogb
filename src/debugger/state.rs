use super::disas::{disas, GbAsm};
use super::metrics::{CpuMetrics, MetricsExport};
use super::CPU_METRICS;
use crate::core::cpu::cpu::{CPURegisters, LR35902CPU};
use crate::core::cpu::interrupts::{INTERRUPT_ENABLE, INTERRUPT_FLAGS};
use crate::core::io::audio::apu::APU;
use crate::core::run_emu::EmuCrash;

pub struct EmuSnapshot {
    pub vram: [u8; 0x2000],
    pub cpu: CpuState,
    pub breakpoints: Vec<u16>,
    pub apu: ApuState,
    pub crash: Option<EmuCrash>,
}

impl Default for EmuSnapshot {
    fn default() -> Self {
        Self {
            vram: [0; 0x2000],
            cpu: CpuState::default(),
            apu: ApuState::default(),
            breakpoints: Vec::new(),
            crash: None,
        }
    }
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
    pub freq: u32,
}

#[derive(Default)]
pub struct ApuState {
    pub channel1: ChannelState,
    pub channel2: ChannelState,
    pub channel3: ChannelState,
    pub channel4: ChannelState,
}

impl ApuState {
    pub fn new(apu: &APU) -> Self {
        Self {
            channel1: ChannelState {
                enabled: apu.channel1.enabled(),
                freq: 131072u32 / (2048u32 - apu.channel1.period as u32),
            },
            channel2: ChannelState {
                enabled: apu.channel2.enabled(),
                freq: 131072u32 / (2048u32 - apu.channel2.period as u32),
            },
            channel3: ChannelState {
                enabled: apu.channel3.enabled(),
                freq: 65536u32 / (2048u32 - apu.channel3.period as u32),
            },
            channel4: ChannelState {
                enabled: apu.channel4.enabled(),
                freq: 0,
            },
        }
    }
}
