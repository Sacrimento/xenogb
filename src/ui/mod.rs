use crate::cpu::cpu::IOEvent;
use crate::io::joypad::JOYPAD_INPUT;
use crate::io::video::ppu::{Vbuf, RESX, RESY};
use cphf::{phf_ordered_map, OrderedMap};
use eframe::egui;
mod debugger;
use crate::debugger::{DebuggerCommand, EmulationState};
use crossbeam_channel::{Receiver, Sender};

static KEYMAP: OrderedMap<u8, egui::Key> = phf_ordered_map! {u8, egui::Key;
    JOYPAD_INPUT::DOWN => egui::Key::ArrowDown,
    JOYPAD_INPUT::UP => egui::Key::ArrowUp,
    JOYPAD_INPUT::LEFT => egui::Key::ArrowLeft,
    JOYPAD_INPUT::RIGHT => egui::Key::ArrowRight,
    JOYPAD_INPUT::A => egui::Key::A,
    JOYPAD_INPUT::B => egui::Key::S,
    JOYPAD_INPUT::SELECT => egui::Key::Space,
    JOYPAD_INPUT::START => egui::Key::Enter,
};
const DEBUGGER_KEY: egui::Key = egui::Key::D;

const SCALE: usize = 4;

pub const WINDOW_SIZE: [f32; 2] = [(RESX * SCALE) as f32, (RESY * SCALE) as f32];

pub struct XenoGBUI {
    screen_buffer: [u8; RESX * RESY * 4],
    screen_texture: egui::TextureHandle,
    debugger: debugger::DebuggerUi,
    video_channel_rc: Receiver<Vbuf>,
    events_sd: Sender<IOEvent>,
    dbg_commands_sd: Sender<DebuggerCommand>,
}

impl XenoGBUI {
    pub fn new(
        ctx: &eframe::CreationContext<'_>,
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

        let debugger = debugger::DebuggerUi::new(ctx, debug, dbg_commands_sd.clone(), dbg_data_rc);

        Self {
            screen_buffer,
            screen_texture,
            debugger,
            video_channel_rc,
            events_sd,
            dbg_commands_sd,
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
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        ctx.input(|inp| {
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

                if inp.viewport().close_requested() {
                    self.events_sd
                        .send(IOEvent::CLOSE)
                        .expect("Could not send close");
                }
            }

            if inp.modifiers.ctrl && inp.key_released(DEBUGGER_KEY) {
                self.debugger.enabled = !self.debugger.enabled;
                self.dbg_commands_sd
                    .send(DebuggerCommand::ENABLED(self.debugger.enabled))
                    .unwrap();
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
