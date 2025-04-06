use super::commands::DebuggerCommand;
use super::state::EmulationState;
use crossbeam_channel::{Receiver, Sender};

use crate::cpu::cpu::LR35902CPU;

pub struct Debugger {
    enabled: bool,

    stepping: bool,
    breakpoints: Vec<u16>,

    ui_commands_rc: Receiver<DebuggerCommand>,
    dbg_data_sd: Sender<EmulationState>,
}

impl Debugger {
    pub fn new(
        enabled: bool,
        ui_commands_rc: Receiver<DebuggerCommand>,
        dbg_data_sd: Sender<EmulationState>,
    ) -> Self {
        Self {
            enabled,
            stepping: false,
            breakpoints: vec![],
            ui_commands_rc,
            dbg_data_sd,
        }
    }

    pub fn handle_events(&mut self) {
        if let Ok(event) = self.ui_commands_rc.try_recv() {
            match event {
                DebuggerCommand::ENABLED(enabled) => self.enabled = enabled,
                _ => todo!(),
            }
        }
    }

    pub fn collect(&self, cpu: &LR35902CPU) {
        if !self.enabled {
            return;
        }

        if self.dbg_data_sd.is_full() {
            return;
        }

        let state = EmulationState {
            vram: cpu.bus.io.ppu.vram.clone(),
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
