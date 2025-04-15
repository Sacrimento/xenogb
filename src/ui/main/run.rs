use super::ui::{XenoGBUI, WINDOW_SIZE};
use crate::core::io::video::ppu::Vbuf;
use crate::core::mem::bus::Bus;
use crate::core::run_emu::run_emu_thread;

use crossbeam_channel::Receiver;
use eframe::egui::ViewportBuilder;

use std::path::PathBuf;

pub fn run_ui(
    bus: Bus,
    video_channel_rc: Receiver<Vbuf>,
    debug: bool,
    serial: bool,
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
            let (emu_state, channels) =
                run_emu_thread(bus, debug, serial, record_enabled, record_path, replay_path);

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
