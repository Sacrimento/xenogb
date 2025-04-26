use crossbeam_channel::{Receiver, Sender};
use eframe::egui;
use egui::{
    Align, CentralPanel, Color32, Key, Layout, RichText, ScrollArea, SidePanel, TextEdit, Ui,
};

use crate::core::run_emu::EmuCrash;
use crate::debugger::{DebuggerCommand, EmuSnapshot, GbAsm};
use crate::ui::debugger::repl::Repl;

pub struct ReplUi {
    dbg_data_rc: Receiver<EmuSnapshot>,

    repl: Repl,
    repl_out: Vec<String>,

    emu_crash: Option<EmuCrash>,

    last_state: EmuSnapshot,
}

impl ReplUi {
    pub fn new(
        dbg_data_rc: Receiver<EmuSnapshot>,
        dbg_commands_sd: Sender<DebuggerCommand>,
    ) -> Self {
        let repl = Repl::new(dbg_commands_sd);

        Self {
            dbg_data_rc,
            repl,
            repl_out: vec![],
            emu_crash: None,
            last_state: EmuSnapshot::default(),
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        if let Ok(data) = self.dbg_data_rc.try_recv() {
            if let Some(crash) = &data.crash {
                self.emu_died(&crash);
            }
            self.last_state = data;
        }

        SidePanel::right("data-panel")
            .min_width(200.0)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                self.disas_ui(ui);
            });

        CentralPanel::default().show(ui.ctx(), |ui| {
            self.repl_ui(ui);
        });
    }

    pub fn emu_died(&mut self, crash: &EmuCrash) {
        self.repl_out
            .push(format!("Emulator crashed at 0x{:04X}", crash.addr));
        self.emu_crash = Some(crash.clone());
        self.repl.emu_died = true;
    }

    fn repl_ui(&mut self, ui: &mut Ui) {
        ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
            let res = TextEdit::singleline(&mut self.repl.cmd)
                .lock_focus(true)
                .desired_width(f32::INFINITY)
                .show(ui);

            ScrollArea::vertical()
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        for line in self.repl_out.iter().rev().take(25).rev() {
                            ui.label(line);
                        }
                    });
                });

            ui.separator();

            if res.response.lost_focus() {
                res.response.request_focus();
            }
            ui.input(|i| {
                if i.key_pressed(Key::Enter) && !self.repl.cmd.is_empty() {
                    self.repl_out.push(format!("> {}", self.repl.cmd));
                    if let Some(_) = self.repl.exec().err() {
                        self.repl_out.push(format!(
                            "Invalid command: {}\nUse help to show available commands",
                            self.repl.cmd
                        ));
                    }
                }
                if i.key_pressed(Key::ArrowUp) {
                    self.repl.history_next();
                }
                if i.key_pressed(Key::ArrowDown) {
                    self.repl.history_prev();
                }
            });
        });
    }

    fn asm_ui(&self, ui: &mut Ui, asm: &GbAsm) {
        let cpu = &self.last_state.cpu;
        let bp = &self.last_state.breakpoints;

        if self.emu_crash.as_ref().is_some_and(|c| c.addr == asm.addr) {
            ui.label(
                RichText::new(format!("{:04X} >>> {}", asm.addr, asm.asm))
                    .monospace()
                    .color(Color32::RED)
                    .strong(),
            );
        } else if bp.contains(&cpu.registers.pc) && cpu.registers.pc == asm.addr {
            ui.label(
                RichText::new(format!("{:04X} >>> {}", asm.addr, asm.asm))
                    .monospace()
                    .strong(),
            );
        } else {
            let mut modifier = "   ";

            if cpu.registers.pc == asm.addr {
                modifier = " > ";
            } else if bp.contains(&asm.addr) {
                modifier = " b ";
            }
            ui.label(RichText::new(format!("{:04X} {modifier} {}", asm.addr, asm.asm)).monospace());
        }
    }

    fn disas_ui(&self, ui: &mut Ui) {
        let cpu = &self.last_state.cpu;

        let last_exec = cpu.disas.first().map_or(0, |asm| asm.addr);
        let next_exec = cpu.disas.get(1).map_or(0, |asm| asm.addr);
        let jumped = last_exec.abs_diff(next_exec) > 3;

        ui.vertical(|ui| {
            for (idx, asm) in cpu.disas.iter().enumerate() {
                if idx == 1 && jumped {
                    ui.label(RichText::new("...").monospace());
                }
                self.asm_ui(ui, asm);
            }
        });
    }
}
