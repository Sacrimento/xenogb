use crate::LR35902CPU;
use std::sync::{Arc, Mutex};

use super::views::vram::Vram;
use eframe::egui;
use egui_tiles;

pub enum Tabs {
    Menu,
    Vram,
}

pub struct DebuggerState {
    pub enabled: bool,

    pub tree: egui_tiles::Tree<Tabs>,

    pub vram: Vram,
}

impl DebuggerState {
    pub fn new(
        ctx: &eframe::CreationContext<'_>,
        enabled: bool,
        cpu: Arc<Mutex<LR35902CPU>>,
    ) -> Self {
        let tabs = vec![Tabs::Vram, Tabs::Menu];
        let vram = Vram::new(ctx, cpu.clone());

        Self {
            enabled,
            tree: egui_tiles::Tree::new_tabs("debugger", tabs),
            vram,
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
