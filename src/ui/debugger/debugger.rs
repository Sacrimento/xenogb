use super::views::{apu::ApuUi, cpu::CpuUi, ppu::PpuUi, repl::ReplUi};
use crate::debugger::{DebuggerCommand, EmuSnapshot};
use crossbeam_channel::{Receiver, Sender};
use eframe::egui;
use egui_tiles;

pub enum Tabs {
    ReplUi,
    Cpu,
    Vram,
    Apu,
}

pub struct DebuggerUi {
    pub enabled: bool,

    pub tree: egui_tiles::Tree<Tabs>,

    pub ppu: PpuUi,
    pub cpu: CpuUi,
    pub repl: ReplUi,
    pub apu: ApuUi,
}

impl DebuggerUi {
    pub fn new(
        ctx: &eframe::CreationContext<'_>,
        enabled: bool,
        dbg_commands_sd: Sender<DebuggerCommand>,
        dbg_data_rc: Receiver<EmuSnapshot>,
    ) -> Self {
        let tabs = vec![Tabs::Apu, Tabs::ReplUi, Tabs::Vram, Tabs::Cpu];
        let ppu = PpuUi::new(ctx, dbg_data_rc.clone());
        let cpu = CpuUi::new(dbg_data_rc.clone(), dbg_commands_sd.clone());
        let repl = ReplUi::new(dbg_data_rc.clone(), dbg_commands_sd.clone());
        let apu = ApuUi::new(dbg_data_rc.clone(), dbg_commands_sd.clone());

        Self {
            enabled,
            tree: egui_tiles::Tree::new_tabs("debugger", tabs),
            ppu,
            cpu,
            repl,
            apu,
        }
    }
}

pub struct Behavior<'a> {
    pub ppu: &'a mut PpuUi,
    pub cpu: &'a mut CpuUi,
    pub repl: &'a mut ReplUi,
    pub apu: &'a mut ApuUi,
}

#[allow(clippy::needless_lifetimes)]
impl<'a> egui_tiles::Behavior<Tabs> for Behavior<'a> {
    fn tab_title_for_pane(&mut self, tab: &Tabs) -> egui::WidgetText {
        match tab {
            Tabs::ReplUi => "REPL".into(),
            Tabs::Vram => "VRAM".into(),
            Tabs::Cpu => "CPU".into(),
            Tabs::Apu => "APU".into(),
        }
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        tab: &mut Tabs,
    ) -> egui_tiles::UiResponse {
        match tab {
            Tabs::ReplUi => self.repl.ui(ui),
            Tabs::Vram => self.ppu.ui(ui),
            Tabs::Cpu => self.cpu.ui(ui),
            Tabs::Apu => self.apu.ui(ui),
        }

        Default::default()
    }
}
