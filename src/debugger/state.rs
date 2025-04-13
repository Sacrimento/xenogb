use super::metrics::{CpuMetrics, MetricsExport};
use super::CPU_METRICS;
use crate::cpu::cpu::{CPURegisters, LR35902CPU};
use crate::cpu::interrupts::{INTERRUPT_ENABLE, INTERRUPT_FLAGS};

pub struct EmuSnapshot {
    pub vram: [u8; 0x2000],
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
