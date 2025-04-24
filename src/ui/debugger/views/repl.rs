use crossbeam_channel::{Receiver, Sender};
use eframe::egui::{self, Color32};
use egui::{Align, CentralPanel, Key, Layout, RichText, ScrollArea, SidePanel, TextEdit, Ui};

use crate::debugger::{CpuState, DebuggerCommand, EmuSnapshot};
use crate::ui::debugger::repl::Repl;

pub struct ReplUi {
    dbg_data_rc: Receiver<EmuSnapshot>,

    repl: Repl,
    repl_out: Vec<String>,
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
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        let cpu_data: CpuState;
        let bp: Vec<u16>;

        if let Ok(data) = self.dbg_data_rc.try_recv() {
            cpu_data = data.cpu;
            bp = data.breakpoints;
        } else {
            return;
        }

        SidePanel::right("data-panel")
            .min_width(200.0)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                self.disas_ui(ui, &cpu_data, bp);
            });

        CentralPanel::default().show(ui.ctx(), |ui| {
            self.repl_ui(ui);
        });
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
                if i.key_pressed(Key::Enter) {
                    self.repl_out.push(format!("> {}", self.repl.cmd));
                    match self.repl.exec() {
                        Ok(_) => (),
                        Err(_) => self.repl_out.push(format!(
                            "Invalid command: {}\nUse help to show available commands",
                            self.repl.cmd
                        )),
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

    fn disas_ui(&self, ui: &mut Ui, cpu_data: &CpuState, breakpoints: Vec<u16>) {
        ui.vertical(|ui| {
            for asm in &cpu_data.disas {
                if breakpoints.contains(&cpu_data.registers.pc) && cpu_data.registers.pc == asm.addr
                {
                    ui.label(
                        RichText::new(format!("{:04X} >>> {}", asm.addr, asm.asm))
                            .monospace()
                            .strong(),
                    );
                } else {
                    let mut modifier = "   ";

                    if cpu_data.registers.pc == asm.addr {
                        modifier = " > ";
                    } else if breakpoints.contains(&asm.addr) {
                        modifier = " b ";
                    }
                    ui.label(
                        RichText::new(format!("{:04X} {modifier} {}", asm.addr, asm.asm))
                            .monospace(),
                    );
                }
            }
        });
    }
}
