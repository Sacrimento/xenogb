use crossbeam_channel::Receiver;
use eframe::egui;
use egui::{Align, Grid, Layout, Ui};

use super::super::utils::timedata::TimeData;
use crate::cpu::cpu::{CPUFlags, CPURegisters};
use crate::cpu::interrupts::InterruptFlags;
use crate::debugger::{
    CpuMetrics, CpuState, EmulationState, InterruptState, MetricType, MetricsExport,
};
use crate::flag_set;

const HISTORY_SIZE: usize = 60;

pub struct CpuUi {
    dbg_data_rc: Receiver<EmulationState>,

    instructions_td: TimeData,
    tick_td: TimeData,
    cycles_td: TimeData,
}

impl CpuUi {
    pub fn new(dbg_data_rc: Receiver<EmulationState>) -> Self {
        Self {
            dbg_data_rc,
            instructions_td: TimeData::new(HISTORY_SIZE, "instructions-dt".into()),
            tick_td: TimeData::new(HISTORY_SIZE, "tick-dt".into()),
            cycles_td: TimeData::new(HISTORY_SIZE, "cycles-dt".into()),
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        let cpu_data: CpuState;

        if let Ok(data) = self.dbg_data_rc.try_recv() {
            cpu_data = data.cpu;
        } else {
            return;
        }

        self.update_td(cpu_data.metrics);

        ui.label(format!(
            "STATE: {}",
            if cpu_data.halt { "HALTED" } else { "RUNNING" }
        ));
        ui.separator();
        self.render_registers(ui, cpu_data.registers);
        ui.separator();
        self.render_interrupts(ui, cpu_data.interrupts);
        ui.separator();
        self.render_metrics(ui, cpu_data.metrics);
    }

    fn render_registers(&mut self, ui: &mut Ui, registers: CPURegisters) {
        Grid::new("cpu-registers-grid").show(ui, |ui| {
            ui.label(format!("A: 0x{:02X}", registers.a));
            ui.label(format!("B: 0x{:02X}", registers.b));
            ui.label(format!("C: 0x{:02X}", registers.c));
            ui.end_row();

            ui.label(format!("D: 0x{:02X}", registers.d));
            ui.label(format!("E: 0x{:02X}", registers.e));
            ui.label(format!("H: 0x{:02X}", registers.h));
            ui.end_row();

            ui.label(format!("L: 0x{:02X}", registers.l));
            ui.label(format!("SP: 0x{:04X}", registers.sp));
            ui.label(format!("PC: 0x{:04X}", registers.pc));
            ui.end_row();

            ui.label(format!(
                "Flags: Z:{} N:{} H:{} C:{}",
                flag_set!(registers.f, CPUFlags::Z) as u8,
                flag_set!(registers.f, CPUFlags::N) as u8,
                flag_set!(registers.f, CPUFlags::H) as u8,
                flag_set!(registers.f, CPUFlags::C) as u8,
            ));
        });
    }

    fn render_interrupts(&mut self, ui: &mut Ui, interrupts_state: InterruptState) {
        ui.label(format!(
            "MASTER INTERRUPT: {}",
            if interrupts_state.int_master {
                "ENABLED"
            } else {
                "DISABLED"
            },
        ));

        ui.label("Enabled interrupts");
        ui.with_layout(Layout::left_to_right(Align::TOP), |ui| {
            ui.label(format!(
                "VBLANK: {}",
                flag_set!(interrupts_state.int_enable, InterruptFlags::VBLANK)
            ));
            ui.label(format!(
                "STAT: {}",
                flag_set!(interrupts_state.int_enable, InterruptFlags::STAT)
            ));
            ui.label(format!(
                "TIMER: {}",
                flag_set!(interrupts_state.int_enable, InterruptFlags::TIMER)
            ));
            ui.label(format!(
                "SERIAL: {}",
                flag_set!(interrupts_state.int_enable, InterruptFlags::SERIAL)
            ));
            ui.label(format!(
                "JOYPAD: {}",
                flag_set!(interrupts_state.int_enable, InterruptFlags::JOYPAD)
            ));
        });
        ui.label("Pending interrupts");
        ui.with_layout(Layout::left_to_right(Align::TOP), |ui| {
            ui.label(format!(
                "VBLANK: {}",
                flag_set!(interrupts_state.interrupts, InterruptFlags::VBLANK)
            ));
            ui.label(format!(
                "STAT: {}",
                flag_set!(interrupts_state.interrupts, InterruptFlags::STAT)
            ));
            ui.label(format!(
                "TIMER: {}",
                flag_set!(interrupts_state.interrupts, InterruptFlags::TIMER)
            ));
            ui.label(format!(
                "SERIAL: {}",
                flag_set!(interrupts_state.interrupts, InterruptFlags::SERIAL)
            ));
            ui.label(format!(
                "JOYPAD: {}",
                flag_set!(interrupts_state.interrupts, InterruptFlags::JOYPAD)
            ));
        });
    }

    fn render_metrics(&mut self, ui: &mut Ui, metrics_export: MetricsExport<CpuMetrics>) {
        ui.with_layout(Layout::left_to_right(Align::TOP), |ui| {
            self.instructions_td.ui(
                ui,
                format!(
                    "Instructions: {} per {:?}",
                    metrics_export.metrics.instructions.get(),
                    metrics_export.duration
                ),
            );
            self.cycles_td.ui(
                ui,
                format!(
                    "Cycles: {} per {:?}",
                    metrics_export.metrics.cycles.get(),
                    metrics_export.duration
                ),
            );
            self.tick_td.ui(
                ui,
                format!(
                    "Average CPU tick time: {:?}",
                    metrics_export.metrics.tick_time.get()
                ),
            );
        });
    }

    fn update_td(&mut self, metrics_export: MetricsExport<CpuMetrics>) {
        self.instructions_td.update(
            metrics_export.at,
            metrics_export.metrics.instructions.get() as f64,
        );
        self.tick_td.update(
            metrics_export.at,
            metrics_export.metrics.tick_time.get().as_secs_f64(),
        );
        self.cycles_td.update(
            metrics_export.at,
            metrics_export.metrics.cycles.get() as f64,
        );
    }
}
