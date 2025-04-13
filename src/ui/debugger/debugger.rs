use super::views::{cpu::CpuUi, vram::VramUi};
use crate::debugger::{DebuggerCommand, EmuSnapshot};
use crossbeam_channel::{Receiver, Sender};
use eframe::egui;
use egui_tiles;

pub enum Tabs {
    Cpu,
    Vram,
}

pub struct DebuggerUi {
    pub enabled: bool,

    pub tree: egui_tiles::Tree<Tabs>,

    pub vram: VramUi,
    pub cpu: CpuUi,
    // dbg_commands_sd: Sender<DebuggerCommand>,
    // dbg_data_rc: Receiver<EmulationState>,
}

impl DebuggerUi {
    pub fn new(
        ctx: &eframe::CreationContext<'_>,
        enabled: bool,
        dbg_commands_sd: Sender<DebuggerCommand>,
        dbg_data_rc: Receiver<EmuSnapshot>,
    ) -> Self {
        let tabs = vec![Tabs::Vram, Tabs::Cpu];
        let vram = VramUi::new(ctx, dbg_data_rc.clone());
        let cpu = CpuUi::new(dbg_data_rc.clone(), dbg_commands_sd.clone());

        Self {
            enabled,
            tree: egui_tiles::Tree::new_tabs("debugger", tabs),
            vram,
            cpu,
            // dbg_commands_sd,
            // dbg_data_rc,
        }
    }
}

pub struct Behavior<'a> {
    pub vram: &'a mut VramUi,
    pub cpu: &'a mut CpuUi,
}

impl<'a> egui_tiles::Behavior<Tabs> for Behavior<'a> {
    fn tab_title_for_pane(&mut self, tab: &Tabs) -> egui::WidgetText {
        match tab {
            Tabs::Vram => "VRAM".into(),
            Tabs::Cpu => "CPU".into(),
        }
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        tab: &mut Tabs,
    ) -> egui_tiles::UiResponse {
        match tab {
            Tabs::Vram => self.vram.ui(ui),
            Tabs::Cpu => self.cpu.ui(ui),
        }

        Default::default()
    }
}
