use super::debugger::DebuggerUi;
use crate::cpu::cpu::LR35902CPU;
use crate::cpu::CLOCK_SPEED;
use crate::debugger::{Debugger, DebuggerCommand, EmulationState};
use crate::io::joypad::JOYPAD_INPUT;
use crate::io::video::ppu::{Vbuf, RESX, RESY};
use crate::io_event::IOEvent;
use crate::io_event::IOListener;
use crate::mem::bus::Bus;
use crate::playback::Playback;
use crate::run;
use cphf::{phf_ordered_map, OrderedMap};
use crossbeam_channel::{bounded, unbounded, Receiver, Sender};
use eframe::egui::{self, Context, Id, Modal};
use egui::{Key, ViewportBuilder, ViewportCommand};
use std::path::PathBuf;
use std::thread::JoinHandle;

static KEYMAP: OrderedMap<u8, Key> = phf_ordered_map! {u8, Key;
    JOYPAD_INPUT::DOWN => Key::ArrowDown,
    JOYPAD_INPUT::UP => Key::ArrowUp,
    JOYPAD_INPUT::LEFT => Key::ArrowLeft,
    JOYPAD_INPUT::RIGHT => Key::ArrowRight,
    JOYPAD_INPUT::A => Key::A,
    JOYPAD_INPUT::B => Key::S,
    JOYPAD_INPUT::SELECT => Key::Space,
    JOYPAD_INPUT::START => Key::Enter,
};
const DEBUGGER_KEY: Key = Key::D;

const SCALE: usize = 4;

pub const WINDOW_SIZE: [f32; 2] = [(RESX * SCALE) as f32, (RESY * SCALE) as f32];

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
            let (ui_debugger_commands_sd, ui_debugger_commands_rc) = unbounded();
            let (dbg_data_sd, dbg_data_rc) = bounded(1);
            let (io_events_sd, io_events_rc) = unbounded();

            let thread = std::thread::spawn(move || {
                run(
                    LR35902CPU::new(bus, serial, CLOCK_SPEED),
                    Debugger::new(debug, ui_debugger_commands_rc, dbg_data_sd),
                    IOListener::new(io_events_rc),
                    Playback::new(record_enabled, record_path, replay_path),
                )
            });

            Ok(Box::new(XenoGBUI::new(
                ctx,
                Some(thread),
                io_events_sd,
                video_channel_rc,
                ui_debugger_commands_sd,
                dbg_data_rc,
                debug,
            )))
        }),
    );
}

struct XenoGBUI {
    screen_buffer: [u8; RESX * RESY * 4],
    screen_texture: egui::TextureHandle,
    debugger: DebuggerUi,
    video_channel_rc: Receiver<Vbuf>,
    events_sd: Sender<IOEvent>,
    dbg_commands_sd: Sender<DebuggerCommand>,
    emu_thread: Option<JoinHandle<()>>,
}

impl XenoGBUI {
    pub fn new(
        ctx: &eframe::CreationContext<'_>,
        emu_thread: Option<JoinHandle<()>>,
        events_sd: Sender<IOEvent>,
        video_channel_rc: Receiver<Vbuf>,
        dbg_commands_sd: Sender<DebuggerCommand>,
        dbg_data_rc: Receiver<EmulationState>,
        debug: bool,
    ) -> Self {
        let screen_buffer = [0xff; RESX * RESY * 4];
        let screen_texture = ctx.egui_ctx.load_texture(
            "screen",
            egui::ColorImage::from_rgba_unmultiplied([RESX, RESY], &screen_buffer),
            egui::TextureOptions::NEAREST,
        );

        let debugger = DebuggerUi::new(ctx, debug, dbg_commands_sd.clone(), dbg_data_rc);

        Self {
            screen_buffer,
            screen_texture,
            debugger,
            video_channel_rc,
            events_sd,
            dbg_commands_sd,
            emu_thread,
        }
    }

    fn render_vbuf(&mut self, ctx: &eframe::egui::Context) {
        if let Ok(vbuf) = self.video_channel_rc.try_recv() {
            for i in 0..vbuf.len() {
                self.screen_buffer[i * 4] = vbuf[i];
                self.screen_buffer[i * 4 + 1] = vbuf[i];
                self.screen_buffer[i * 4 + 2] = vbuf[i];
            }

            ctx.tex_manager().write().set(
                self.screen_texture.id(),
                egui::epaint::ImageDelta::full(
                    egui::ColorImage::from_rgba_unmultiplied([RESX, RESY], &self.screen_buffer),
                    egui::TextureOptions::NEAREST,
                ),
            );
        }
    }
}

impl eframe::App for XenoGBUI {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        if self.emu_thread.as_ref().is_none_or(|t| t.is_finished()) {
            emulation_died(self.emu_thread.take().map(|t| t.join()), ctx);
        }

        ctx.input(|inp| {
            if self.emu_thread.is_none() {
                return;
            }

            for (emu_key, ui_key) in KEYMAP.entries() {
                if inp.key_pressed(*ui_key) {
                    self.events_sd
                        .send(IOEvent::JOYPAD_PRESS(*emu_key))
                        .expect("Could not send ui key press");
                }
                if inp.key_released(*ui_key) {
                    self.events_sd
                        .send(IOEvent::JOYPAD_RELEASE(*emu_key))
                        .expect("Could not send ui key release");
                }
            }

            if inp.viewport().close_requested() {
                self.events_sd
                    .send(IOEvent::CLOSE)
                    .expect("Could not send close");
            }

            if inp.modifiers.ctrl && inp.key_released(DEBUGGER_KEY) {
                self.debugger.enabled = !self.debugger.enabled;
                self.dbg_commands_sd
                    .send(DebuggerCommand::ENABLED(self.debugger.enabled))
                    .expect("Could not send dbg command");
            }
        });

        self.render_vbuf(ctx);

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let screen = egui::Image::from_texture(&self.screen_texture)
                    .fit_to_original_size(SCALE as f32)
                    .maintain_aspect_ratio(true);
                ui.add(screen);
            });

        if self.debugger.enabled {
            self.debugger.ui(ctx);
        }

        ctx.request_repaint();
    }
}

fn emulation_died(res: Option<Result<(), Box<dyn std::any::Any + Send>>>, ctx: &Context) {
    if let Some(res) = res {
        println!(
            "That sucks! {:#?}",
            res.err().unwrap().downcast::<&'static str>().ok().unwrap()
        );
    }

    Modal::new(Id::new("crash-popup")).show(ctx, |ui| {
        ui.label("Emulation crashed :(");
        if ui.button("Ok").clicked() {
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
    });
}
