use super::{Debugger, CPU_METRICS};
use crate::{
    core::cpu::{instructions::CPURegisterId, LR35902CPU},
    debugger::PPU_METRICS,
};

use log::info;

#[derive(Clone, Copy, Debug, Default)]
pub struct DynAddr {
    addr: Option<u16>,
    reg: Option<CPURegisterId>,
}

impl DynAddr {
    pub fn new(addr: Option<u16>, reg: Option<CPURegisterId>) -> Self {
        Self { addr, reg }
    }

    pub fn resolve(&self, cpu: &LR35902CPU) -> u16 {
        if let Some(addr) = self.addr {
            return addr;
        }

        if let Some(reg) = &self.reg {
            if reg > &CPURegisterId::L {
                return cpu.get_register16(reg);
            }
            return cpu.get_register(reg) as u16;
        }
        0
    }
}

#[allow(nonstandard_style)]
#[derive(Debug)]
pub enum DebuggerCommand {
    ENABLED(bool),

    CPU_CLOCK(u32),

    APU_MUTE_CHANNEL(u8),

    STEP,
    CONTINUE,
    BREAKPOINT(DynAddr),
}

impl Debugger {
    pub fn breakpoint(&mut self, addr: u16) {
        if let Some(i) = self.breakpoints.iter().position(|a| *a == addr) {
            self.breakpoints.remove(i);
        } else {
            info!("Added breakpoint at 0x{:04X}", addr);
            self.breakpoints.push(addr);
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        info!("Debugger is now enabled:{enabled}");
        self.enabled = enabled;

        CPU_METRICS.with_borrow_mut(|mh| mh.set_enabled(enabled));
        PPU_METRICS.with_borrow_mut(|mh| mh.set_enabled(enabled));
    }
}
