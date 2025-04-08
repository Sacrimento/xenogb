use eframe::egui;
use egui::{Sense, Ui};
use egui_plot::{Line, Plot, PlotPoints};
use std::{collections::VecDeque, time::Instant};

pub struct TimeData {
    id: String,
    history: VecDeque<f64>,
    history_size: usize,

    last_update: Instant,
}

impl TimeData {
    pub fn new(history_size: usize, id: String) -> Self {
        Self {
            id,
            history_size,
            history: VecDeque::with_capacity(history_size),
            last_update: Instant::now(),
        }
    }

    pub fn ui(&self, ui: &mut Ui, text: String) {
        let history = self.history.iter().copied().collect::<Vec<f64>>();
        let points: PlotPoints = PlotPoints::from_ys_f64(history.as_slice());
        let line = Line::new(points);

        Plot::new(self.id.as_str())
            .height(300.0)
            .width(400.0)
            .show_background(true)
            .show_grid(false)
            .sense(Sense::empty())
            .allow_scroll(false)
            .allow_zoom(false)
            .clamp_grid(true)
            .show_axes([true, false])
            .x_axis_label(text)
            .show(ui, |plot_ui| plot_ui.line(line.highlight(true)));
    }

    pub fn update(&mut self, at: Instant, value: f64) {
        if at > self.last_update {
            if self.history.len() == self.history_size {
                self.history.pop_front();
            }
            self.history.push_back(value);
            self.last_update = at;
        }
    }
}
