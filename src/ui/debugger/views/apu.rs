use crossbeam_channel::{Receiver, Sender};
use eframe::egui::{self, Color32};
use egui::{RichText, Sense, Ui};
use egui_plot::{Legend, Line, Plot, PlotPoints};
use itertools::Itertools;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::debugger::{ApuState, DebuggerCommand, EmuSnapshot};

const CHANNEL_FREQS_UPDATE_INTERVAL: Duration = Duration::from_millis(5);
const CHANNEL_TIME_HISTORY: Duration = Duration::from_secs(15);
const CHANNEL_COLORS: [Color32; 4] = [Color32::RED, Color32::YELLOW, Color32::GREEN, Color32::GRAY];
const NOTE_NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

pub struct ApuUi {
    channel1_cb: bool,
    channel2_cb: bool,
    channel3_cb: bool,
    channel4_cb: bool,

    channel_freqs: ChannelFreqs,

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
            channel_freqs: ChannelFreqs::new(
                (CHANNEL_TIME_HISTORY.as_millis() / CHANNEL_FREQS_UPDATE_INTERVAL.as_millis())
                    as usize,
                CHANNEL_FREQS_UPDATE_INTERVAL,
            ),
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

        self.channel_freqs.update(&apu_data);

        self.channel_enable_ui(ui);
        self.channel_freqs.ui(ui);
    }

    fn channel_enable_ui(&mut self, ui: &mut Ui) {
        ui.label("Channel Enable");
        ui.horizontal(|ui| {
            let mut cb_callback = |channel_b: &mut bool, channel_s: RichText, channel_n| {
                let res = ui.checkbox(channel_b, channel_s);
                if res.changed() {
                    _ = self
                        .dbg_commands_sd
                        .send(DebuggerCommand::APU_MUTE_CHANNEL(channel_n))
                        .unwrap();
                }
            };

            cb_callback(
                &mut self.channel1_cb,
                RichText::new("Channel 1").color(CHANNEL_COLORS[0]),
                1,
            );
            cb_callback(
                &mut self.channel2_cb,
                RichText::new("Channel 2").color(CHANNEL_COLORS[1]),
                2,
            );
            cb_callback(
                &mut self.channel3_cb,
                RichText::new("Channel 3").color(CHANNEL_COLORS[2]),
                3,
            );
            cb_callback(
                &mut self.channel4_cb,
                RichText::new("Channel 4").color(CHANNEL_COLORS[3]),
                4,
            );
        });
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

        Plot::new("channel-freqs")
            // .sense(Sense::empty())
            // .allow_scroll(false)
            // .allow_zoom(false)
            // .clamp_grid(true)
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
                if channel.muted || !channel.enabled || channel.volume == 0 {
                    self.history[i].push_back(None);
                } else {
                    self.history[i].push_back(Some(channel.freq));
                }
            }
            self.last_update = Instant::now();
        }
    }
}
