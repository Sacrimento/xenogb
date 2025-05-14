use crossbeam_channel::{Receiver, Sender};
use eframe::egui;
use egui::{Sense, Ui};
use egui_plot::{Line, Plot, PlotPoints};

use crate::debugger::{ApuState, DebuggerCommand, EmuSnapshot};

pub struct ApuUi {
    dbg_data_rc: Receiver<EmuSnapshot>,
    _dbg_commands_sd: Sender<DebuggerCommand>,
}

impl ApuUi {
    pub fn new(
        dbg_data_rc: Receiver<EmuSnapshot>,
        _dbg_commands_sd: Sender<DebuggerCommand>,
    ) -> Self {
        Self {
            dbg_data_rc,
            _dbg_commands_sd,
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        let apu_data: ApuState;

        if let Ok(data) = self.dbg_data_rc.try_recv() {
            apu_data = data.apu;
        } else {
            return;
        }

        self.spectogram_ui(ui, &apu_data);
    }

    fn spectogram_ui(&mut self, ui: &mut Ui, apu_data: &ApuState) {
        let mut points: Vec<f32> = vec![];
        points.resize(20000, 0.0);

        points[apu_data.channel1.freq.floor() as usize] = 1.0;
        points[apu_data.channel2.freq.floor() as usize] = 1.0;
        points[apu_data.channel3.freq.floor() as usize] = 1.0;
        points[apu_data.channel4.freq.floor() as usize] = 1.0;

        let l = Line::new(PlotPoints::from_ys_f32(points.as_slice()));

        ui.label("Spectrogram");
        Plot::new("skibidiplot")
            .sense(Sense::empty())
            .allow_scroll(false)
            .allow_zoom(false)
            .clamp_grid(true)
            .show(ui, |plot_ui| plot_ui.line(l.highlight(true)));
    }
}
