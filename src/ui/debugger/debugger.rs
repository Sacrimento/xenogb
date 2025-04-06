use super::views::vram::Vram;
use crate::debugger::{DebuggerCommand, EmulationState};
use crossbeam_channel::{Receiver, Sender};
use eframe::egui;
use egui_tiles;

pub enum Tabs {
    Menu,
    Vram,
}

pub struct DebuggerUi {
    pub enabled: bool,

    pub tree: egui_tiles::Tree<Tabs>,

    pub vram: Vram,
    // dbg_commands_sd: Sender<DebuggerCommand>,
    // dbg_data_rc: Receiver<EmulationState>,
}

impl DebuggerUi {
    pub fn new(
        ctx: &eframe::CreationContext<'_>,
        enabled: bool,
        _dbg_commands_sd: Sender<DebuggerCommand>,
        dbg_data_rc: Receiver<EmulationState>,
    ) -> Self {
        let tabs = vec![Tabs::Vram, Tabs::Menu];
        let vram = Vram::new(ctx, dbg_data_rc.clone());

        Self {
            enabled,
            tree: egui_tiles::Tree::new_tabs("debugger", tabs),
            vram,
            // dbg_commands_sd,
            // dbg_data_rc,
        }
    }
}

pub struct Behavior<'a> {
    pub vram: &'a mut Vram,
}

impl<'a> egui_tiles::Behavior<Tabs> for Behavior<'a> {
    fn tab_title_for_pane(&mut self, tab: &Tabs) -> egui::WidgetText {
        match tab {
            Tabs::Vram => "VRAM".into(),
            Tabs::Menu => "MENU".into(),
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
            Tabs::Menu => {
                ui.label("menu");
            }
        }

        Default::default()
    }
}
