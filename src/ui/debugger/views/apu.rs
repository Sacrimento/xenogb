use crossbeam_channel::{Receiver, Sender};
use eframe::egui::{Color32, Pos2, Rect, Sense, Separator, Slider, Stroke, StrokeKind, Ui, Vec2};
use egui_plot::{Legend, Line, Plot, PlotPoints};
use itertools::Itertools;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

use super::super::utils::Cache;
use crate::debugger::{ApuState, DebuggerCommand, EmuSnapshot};

const CHANNEL_FREQS_UPDATE_INTERVAL: Duration = Duration::from_millis(5);
const CHANNEL_TIME_HISTORY: Duration = Duration::from_secs(3);
const CHANNEL_COLORS: [Color32; 4] = [Color32::RED, Color32::YELLOW, Color32::GREEN, Color32::GRAY];
const NOTE_NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

pub struct ApuUi {
    channel_freqs: ChannelFreqs,

    mixer: Mixer,

    dbg_data_rc: Cache,
}

impl ApuUi {
    pub fn new(
        dbg_data_rc: Receiver<EmuSnapshot>,
        dbg_commands_sd: Sender<DebuggerCommand>,
    ) -> Self {
        Self {
            channel_freqs: ChannelFreqs::new(
                (CHANNEL_TIME_HISTORY.as_millis() / CHANNEL_FREQS_UPDATE_INTERVAL.as_millis())
                    as usize,
                CHANNEL_FREQS_UPDATE_INTERVAL,
            ),
            mixer: Mixer::new(dbg_commands_sd),
            dbg_data_rc: Cache::new(dbg_data_rc),
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        let apu_data = self.dbg_data_rc.get().apu;

        self.channel_freqs.update(&apu_data);

        self.channel_freqs.ui(ui);
        self.mixer.ui(ui, &apu_data);
    }
}

struct Mixer {
    channel1_volume: f32,
    channel1_enabled: bool,

    channel2_volume: f32,
    channel2_enabled: bool,

    channel3_volume: f32,
    channel3_enabled: bool,

    channel4_volume: f32,
    channel4_enabled: bool,

    left_volume: f32,
    right_volume: f32,
    master_volume: f32,

    dbg_commands_sd: Sender<DebuggerCommand>,
}

impl Mixer {
    pub fn new(sender: Sender<DebuggerCommand>) -> Self {
        Self {
            channel1_volume: 1.0,
            channel1_enabled: true,
            channel2_volume: 1.0,
            channel2_enabled: true,
            channel3_volume: 1.0,
            channel3_enabled: true,
            channel4_volume: 1.0,
            channel4_enabled: true,
            left_volume: 1.0,
            right_volume: 1.0,
            master_volume: 1.0,
            dbg_commands_sd: sender,
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, apu_data: &ApuState) {
        let cb_callback = |ui: &mut Ui, channel_b: &mut bool, channel_n| {
            let res = ui.checkbox(channel_b, "");
            if res.changed() {
                _ = self
                    .dbg_commands_sd
                    .send(DebuggerCommand::APU_CHANNEL_MUTE(channel_n));
            }
        };

        let slider_callback = |ui: &mut Ui, vol: &mut f32, text: &str| {
            let res = ui.add(
                Slider::new(vol, 0.0..=2.0)
                    .vertical()
                    .show_value(false)
                    .text(text),
            );
            res.lost_focus() || res.dragged()
        };

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                if slider_callback(ui, &mut self.channel1_volume, "CH 1") {
                    _ = self
                        .dbg_commands_sd
                        .send(DebuggerCommand::APU_CHANNEL_VOLUME((
                            1,
                            self.channel1_volume,
                        )));
                }
                cb_callback(ui, &mut self.channel1_enabled, 1);
            });
            Self::vu_meter_ui(
                ui,
                (apu_data.channel1.volume as f32 / 15.0) * self.channel1_volume,
                Vec2::new(6.0, ui.available_height()),
            );

            ui.vertical(|ui| {
                if slider_callback(ui, &mut self.channel2_volume, "CH 2") {
                    _ = self
                        .dbg_commands_sd
                        .send(DebuggerCommand::APU_CHANNEL_VOLUME((
                            2,
                            self.channel2_volume,
                        )));
                }
                cb_callback(ui, &mut self.channel2_enabled, 2);
            });
            Self::vu_meter_ui(
                ui,
                (apu_data.channel2.volume as f32 / 15.0) * self.channel2_volume,
                Vec2::new(6.0, ui.available_height()),
            );

            ui.vertical(|ui| {
                if slider_callback(ui, &mut self.channel3_volume, "CH 3") {
                    _ = self
                        .dbg_commands_sd
                        .send(DebuggerCommand::APU_CHANNEL_VOLUME((
                            3,
                            self.channel3_volume,
                        )));
                }
                cb_callback(ui, &mut self.channel3_enabled, 3);
            });
            Self::vu_meter_ui(
                ui,
                (apu_data.channel3.volume as f32 / 15.0) * self.channel3_volume,
                Vec2::new(6.0, ui.available_height()),
            );

            ui.vertical(|ui| {
                if slider_callback(ui, &mut self.channel4_volume, "CH 4") {
                    _ = self
                        .dbg_commands_sd
                        .send(DebuggerCommand::APU_CHANNEL_VOLUME((
                            4,
                            self.channel4_volume,
                        )));
                }
                cb_callback(ui, &mut self.channel4_enabled, 4);
            });
            Self::vu_meter_ui(
                ui,
                (apu_data.channel4.volume as f32 / 15.0) * self.channel4_volume,
                Vec2::new(6.0, ui.available_height()),
            );

            ui.add(Separator::default().vertical());

            let master_volume = ((apu_data.channel1.volume as f32 / 15.0)
                + ((apu_data.channel2.volume as f32) / 15.0)
                + (apu_data.channel3.volume as f32 / 15.0)
                + (apu_data.channel4.volume as f32 / 15.0))
                / 4.0;
            if slider_callback(ui, &mut self.left_volume, "L") {
                _ = self
                    .dbg_commands_sd
                    .send(DebuggerCommand::APU_VOLUME_LEFT(self.left_volume));
            }
            Self::vu_meter_ui(
                ui,
                master_volume
                    * ((apu_data.master_volume.left as f32 + 1.) / 16.0)
                    * self.left_volume,
                Vec2::new(6.0, ui.available_height()),
            );
            if slider_callback(ui, &mut self.right_volume, "R") {
                _ = self
                    .dbg_commands_sd
                    .send(DebuggerCommand::APU_VOLUME_RIGHT(self.right_volume));
            }
            Self::vu_meter_ui(
                ui,
                master_volume
                    * ((apu_data.master_volume.right as f32 + 1.) / 16.0)
                    * self.right_volume,
                Vec2::new(6.0, ui.available_height()),
            );
            ui.add(Separator::default().vertical());

            if slider_callback(ui, &mut self.master_volume, "M") {
                _ = self
                    .dbg_commands_sd
                    .send(DebuggerCommand::APU_VOLUME(self.master_volume));
            }
            Self::vu_meter_ui(
                ui,
                master_volume
                    * ((apu_data.master_volume.left as f32 + 1.) / 16.0)
                    * ((apu_data.master_volume.right as f32 + 1.) / 16.0)
                    * self.master_volume,
                Vec2::new(6.0, ui.available_height()),
            );
        });
    }

    pub fn vu_meter_ui(ui: &mut Ui, level: f32, size: Vec2) {
        let level = level.clamp(0.0, 1.0);

        let (_, rect) = ui.allocate_space(size);
        let painter = ui.painter_at(rect);

        painter.rect_filled(rect, 2.0, Color32::from_gray(25));

        let filled_h = rect.height() * level;
        let filled_rect =
            Rect::from_min_max(Pos2::new(rect.min.x, rect.max.y - filled_h), rect.max);

        let fill = if level < 0.60 {
            Color32::from_rgb(0, 200, 0)
        } else if level < 0.85 {
            Color32::from_rgb(255, 200, 0)
        } else {
            Color32::from_rgb(255, 64, 0)
        };

        painter.rect_filled(filled_rect, 2.0, fill);
        painter.rect_stroke(
            rect,
            2.0,
            Stroke::new(1.0, Color32::from_gray(80)),
            StrokeKind::Middle,
        );
    }
}

pub struct ChannelFreqs {
    history: [VecDeque<Option<u32>>; 4],
    history_size: usize,
    refresh_interval: Duration,

    last_update: Instant,
}

impl ChannelFreqs {
    pub fn new(history_size: usize, refresh_interval: Duration) -> Self {
        Self {
            history_size,
            history: [
                VecDeque::with_capacity(history_size),
                VecDeque::with_capacity(history_size),
                VecDeque::with_capacity(history_size),
                VecDeque::with_capacity(history_size),
            ],
            refresh_interval,
            last_update: Instant::now(),
        }
    }

    pub fn ui(&self, ui: &mut Ui) {
        let mut lines: [Vec<Line>; 4] = [vec![], vec![], vec![], vec![]];

        for i in 0..4 {
            let mut idx = 0;

            for (is_some, chunk) in &self.history[i].iter().chunk_by(|v| v.is_some()) {
                if is_some {
                    for (_, c) in &chunk.chunk_by(|&v| v) {
                        let ys = c.map(|v| v.unwrap() as f64).collect::<Vec<f64>>();
                        let chunk_len = ys.len();
                        let points = PlotPoints::new(
                            ys.iter()
                                .zip_eq(idx..idx + chunk_len)
                                .map(|(y, x)| [x as f64, *y])
                                .collect::<Vec<[f64; 2]>>(),
                        );
                        idx += chunk_len;
                        let line = Line::new(points).color(CHANNEL_COLORS[i]);
                        lines[i].push(line);
                    }
                } else {
                    idx += chunk.count();
                }
            }
        }

        ui.collapsing("channel-freqs", |ui| {
            Plot::new("channel-freqs")
                .sense(Sense::empty())
                .allow_scroll(false)
                .allow_zoom(false)
                .clamp_grid(true)
                .legend(Legend::default())
                .show_grid([false, false])
                .label_formatter(|_, value| {
                    let note_number = (12.0 * (value.y / 440.0).log2() + 69.0).round() as u64;

                    let name = NOTE_NAMES[(note_number % 12) as usize];
                    let octave = (note_number / 12).saturating_sub(1);

                    format!("{name}{octave}")
                })
                .show(ui, |plot_ui| {
                    for channel in lines {
                        for line in channel {
                            plot_ui.line(line);
                        }
                    }
                });
        });
    }

    pub fn update(&mut self, apu_state: &ApuState) {
        if self.last_update.elapsed() > self.refresh_interval {
            for (i, channel) in [
                &apu_state.channel1,
                &apu_state.channel2,
                &apu_state.channel3,
                &apu_state.channel4,
            ]
            .iter()
            .enumerate()
            {
                if self.history[i].len() == self.history_size {
                    self.history[i].pop_front();
                }
                if !channel.enabled || channel.volume == 0 {
                    self.history[i].push_back(None);
                } else {
                    self.history[i].push_back(Some(channel.freq));
                }
            }
            self.last_update = Instant::now();
        }
    }
}
