use crossbeam_channel::{Receiver, Sender};
use eframe::egui;
use egui::{Sense, Ui};
use egui_plot::{Line, Plot, PlotPoints};

use crate::debugger::{ApuState, DebuggerCommand, EmuSnapshot};

pub struct ApuUi {
    channel1_cb: bool,
    channel2_cb: bool,
    channel3_cb: bool,
    channel4_cb: bool,

    dbg_data_rc: Receiver<EmuSnapshot>,
    dbg_commands_sd: Sender<DebuggerCommand>,
}

impl ApuUi {
    pub fn new(
        dbg_data_rc: Receiver<EmuSnapshot>,
        dbg_commands_sd: Sender<DebuggerCommand>,
    ) -> Self {
        Self {
            channel1_cb: true,
            channel2_cb: true,
            channel3_cb: true,
            channel4_cb: true,
            dbg_data_rc,
            dbg_commands_sd,
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        let apu_data: ApuState;

        if let Ok(data) = self.dbg_data_rc.try_recv() {
            apu_data = data.apu;
        } else {
            return;
        }

        self.channel_enable_ui(ui);
        self.spectogram_ui(ui, &apu_data);
    }

    fn channel_enable_ui(&mut self, ui: &mut Ui) {
        ui.label("Channel Enable");
        ui.horizontal(|ui| {
            let mut cb_callback = |channel_b: &mut bool, channel_s: &'static str, channel_n| {
                let res = ui.checkbox(channel_b, channel_s);
                if res.changed() {
                    _ = self
                        .dbg_commands_sd
                        .send(DebuggerCommand::APU_MUTE_CHANNEL(channel_n))
                        .unwrap();
                }
            };

            cb_callback(&mut self.channel1_cb, "Channel 1", 1);
            cb_callback(&mut self.channel2_cb, "Channel 2", 2);
            cb_callback(&mut self.channel3_cb, "Channel 3", 3);
            cb_callback(&mut self.channel4_cb, "Channel 4", 4);
        });
    }

    fn spectogram_ui(&mut self, ui: &mut Ui, apu_data: &ApuState) {
        let mut points: Vec<f32> = vec![];
        points.resize(5000, 0.0);

        points[apu_data.channel1.freq.clamp(0, 4999) as usize] = 1.0;
        points[apu_data.channel2.freq.clamp(0, 4999) as usize] = 1.0;
        points[apu_data.channel3.freq.clamp(0, 4999) as usize] = 1.0;
        points[apu_data.channel4.freq.clamp(0, 4999) as usize] = 1.0;

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
