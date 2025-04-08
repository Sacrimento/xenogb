use super::metrics::{CpuMetrics, MetricsHandler};
use super::state::EmulationState;
use super::{commands::DebuggerCommand, state::CpuState};
use crossbeam_channel::{Receiver, Sender};
use std::time::Duration;

use crate::cpu::cpu::LR35902CPU;
use std::cell::RefCell;

thread_local! {
    pub static CPU_METRICS: RefCell<MetricsHandler<CpuMetrics>> = RefCell::new(MetricsHandler::<CpuMetrics>::new(Duration::from_millis(500)));
}

pub struct Debugger {
    enabled: bool,

    // stepping: bool,
    // breakpoints: Vec<u16>,
    ui_commands_rc: Receiver<DebuggerCommand>,
    dbg_data_sd: Sender<EmulationState>,
}

impl Debugger {
    pub fn new(
        enabled: bool,
        ui_commands_rc: Receiver<DebuggerCommand>,
        dbg_data_sd: Sender<EmulationState>,
    ) -> Self {
        CPU_METRICS.with_borrow_mut(|mh| mh.set_enabled(enabled));

        Self {
            enabled,
            // stepping: false,
            // breakpoints: vec![],
            ui_commands_rc,
            dbg_data_sd,
        }
    }

    pub fn handle_events(&mut self) {
        if let Ok(event) = self.ui_commands_rc.try_recv() {
            match event {
                DebuggerCommand::ENABLED(enabled) => self.set_enabled(enabled),
            }
        }
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;

        CPU_METRICS.with_borrow_mut(|mh| mh.set_enabled(enabled));
    }

    pub fn collect(&self, cpu: &LR35902CPU) {
        if !self.enabled {
            return;
        }

        if self.dbg_data_sd.is_full() {
            return;
        }

        CPU_METRICS.with_borrow_mut(|mh| mh.update());

        let state = EmulationState {
            vram: cpu.bus.io.ppu.vram.clone(),
            cpu: CpuState::new(cpu),
        };

        self.dbg_data_sd
            .send(state)
            .expect("Failed to send emulation state");
    }

    pub fn cpu_should_step(&mut self, _cpu: &LR35902CPU) -> bool {
        if !self.enabled {
            return true;
        }

        // Stepping conditions, breakpoints, etc

        return true;
    }
}
