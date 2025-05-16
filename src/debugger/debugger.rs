use super::metrics::{CpuMetrics, MetricsHandler};
use super::state::EmuSnapshot;
use super::{commands::DebuggerCommand, state::ApuState, state::CpuState};
use crate::core::cpu::cpu::LR35902CPU;
use crate::core::run_emu::EmuCrash;
use crossbeam_channel::{Receiver, Sender};

use std::cell::RefCell;
use std::time::Duration;

thread_local! {
    pub static CPU_METRICS: RefCell<MetricsHandler<CpuMetrics>> = RefCell::new(MetricsHandler::<CpuMetrics>::new(Duration::from_millis(1000)));
}

pub struct Debugger {
    pub enabled: bool,

    pub breakpoints: Vec<u16>,
    pub stepping: bool,
    do_step: bool,
    resume: bool,

    pub executing_pc: u16,

    ui_commands_rc: Receiver<DebuggerCommand>,
    dbg_data_sd: Sender<EmuSnapshot>,
}

impl Debugger {
    pub fn new(
        enabled: bool,
        ui_commands_rc: Receiver<DebuggerCommand>,
        dbg_data_sd: Sender<EmuSnapshot>,
    ) -> Self {
        CPU_METRICS.with_borrow_mut(|mh| mh.set_enabled(enabled));

        Self {
            enabled,
            stepping: false,
            do_step: false,
            resume: false,
            breakpoints: vec![],
            executing_pc: 0,
            ui_commands_rc,
            dbg_data_sd,
        }
    }

    pub fn handle_events(&mut self, cpu: &mut LR35902CPU) {
        if let Ok(event) = self.ui_commands_rc.try_recv() {
            match event {
                DebuggerCommand::ENABLED(enabled) => self.set_enabled(enabled),
                DebuggerCommand::CPU_CLOCK(clock_speed) => cpu.clock.set_speed(clock_speed),
                DebuggerCommand::APU_MUTE_CHANNEL(ch_num) => match ch_num {
                    1 => cpu.bus.io.apu.channel1.mute(),
                    2 => cpu.bus.io.apu.channel2.mute(),
                    3 => cpu.bus.io.apu.channel3.mute(),
                    4 => cpu.bus.io.apu.channel4.mute(),
                    _ => unreachable!(),
                },
                DebuggerCommand::CONTINUE => self.resume = true,
                DebuggerCommand::STEP => {
                    if self.stepping {
                        self.do_step = true;
                    }
                }
                DebuggerCommand::BREAKPOINT(addr) => self.breakpoint(addr.resolve(cpu)),
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

        CPU_METRICS.with_borrow_mut(|mh| mh.update());

        let state = EmuSnapshot {
            vram: cpu.bus.io.ppu.vram,
            cpu: CpuState::new(cpu, self.executing_pc),
            apu: ApuState::new(&cpu.bus.io.apu),
            breakpoints: self.breakpoints.clone(),
            ..Default::default()
        };

        self.dbg_data_sd
            .send(state)
            .expect("Failed to send emulation state");
    }

    pub fn cpu_should_step(&mut self, cpu: &LR35902CPU) -> bool {
        if !self.enabled {
            return true;
        }

        if self.resume {
            self.resume = false;
            self.stepping = false;
            return true;
        }

        if self.stepping {
            if self.do_step {
                self.do_step = false;
                return true;
            }
            return false;
        }

        if self.breakpoints.contains(&cpu.pc()) {
            self.stepping = true;
            return false;
        }

        true
    }

    pub fn died(&mut self, cpu: &LR35902CPU, emu_crash: EmuCrash) {
        if !self.enabled {
            return;
        }

        self.dbg_data_sd
            .send(EmuSnapshot {
                vram: cpu.bus.io.ppu.vram,
                cpu: CpuState::new(cpu, self.executing_pc),
                apu: ApuState::new(&cpu.bus.io.apu),
                breakpoints: self.breakpoints.clone(),
                crash: Some(emu_crash),
            })
            .expect("Failed to send emulation state");
    }
}
