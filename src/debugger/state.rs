use super::disas::{disas, GbAsm};
use super::metrics::{CpuMetrics, MetricsExport};
use super::CPU_METRICS;
use crate::core::cpu::cpu::{CPURegisters, LR35902CPU};
use crate::core::cpu::interrupts::{INTERRUPT_ENABLE, INTERRUPT_FLAGS};

pub struct EmuSnapshot {
    pub vram: [u8; 0x2000],
    pub cpu: CpuState,
    pub breakpoints: Vec<u16>,
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
    pub disas: Vec<GbAsm>,
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
            disas: disas(cpu, 30),
        }
    }
}
