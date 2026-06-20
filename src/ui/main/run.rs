use super::ui::{XenoGBUI, WINDOW_SIZE};
use crate::audio::run_audio::run_audio_thread;
use crate::core::io::video::ppu::Vbuf;
use crate::core::mem::bus::Bus;
use crate::core::run_emu::run_emu_thread;

use crossbeam_channel::Receiver;
use eframe::egui::ViewportBuilder;
use crate::core::cpu::CPUSpeed;

use std::path::PathBuf;

#[allow(clippy::too_many_arguments)]
pub fn run_ui(
    bus: Bus,
    video_channel_rc: Receiver<Vbuf>,
    audio_channel_rc: Receiver<[f32; 2]>,
    debug: bool,
    serial: bool,
    cpu_speed: CPUSpeed,
    record_enabled: bool,
    record_path: Option<PathBuf>,
    replay_path: Option<PathBuf>,
) {
    let _ = eframe::run_native(
        "xenogb",
        eframe::NativeOptions {
            viewport: ViewportBuilder::default()
                .with_inner_size(WINDOW_SIZE)
                .with_resizable(false),
            ..Default::default()
        },
        Box::new(move |ctx| {
            run_audio_thread(audio_channel_rc);

            let (emu_state, channels) = run_emu_thread(
                bus,
                debug,
                serial,
                cpu_speed,
                record_enabled,
                record_path,
                replay_path,
            );

            Ok(Box::new(XenoGBUI::new(
                ctx,
                emu_state,
                channels.0,
                video_channel_rc,
                channels.1,
                channels.2,
                debug,
            )))
        }),
    );
}
