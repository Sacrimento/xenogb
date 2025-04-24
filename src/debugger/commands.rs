use super::{Debugger, CPU_METRICS};

use log::info;

#[allow(nonstandard_style)]
#[derive(Debug)]
pub enum DebuggerCommand {
    ENABLED(bool),

    CPU_CLOCK(u32),

    STEP,
    CONTINUE,
    BREAKPOINT(u16),
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
    }
}
