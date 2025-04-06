use super::super::{Behavior, DebuggerUi};

use eframe;
use eframe::egui;

impl DebuggerUi {
    pub fn ui(&mut self, ctx: &egui::Context) {
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("debugger"),
            egui::ViewportBuilder::default()
                .with_title("Debugger")
                .with_inner_size([800.0, 600.0]),
            |ctx, _| {
                egui::CentralPanel::default()
                    .frame(egui::Frame::NONE)
                    .show(ctx, |ui| {
                        let mut behavior = Behavior {
                            vram: &mut self.vram,
                        };
                        self.tree.ui(&mut behavior, ui);
                    });

                if ctx.input(|i| i.viewport().close_requested()) {
                    self.enabled = false;
                }
            },
        );
    }
}
