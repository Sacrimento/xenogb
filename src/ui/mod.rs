use crate::io::joypad::JOYPAD_INPUT;
use crate::io::video::ppu::{Vbuf, RESX, RESY};
use crate::LR35902CPU;
use cphf::{phf_ordered_map, OrderedMap};
use eframe::egui;
use std::sync::{Arc, Mutex};
mod debugger;
use crossbeam_channel::Receiver;

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
    cpu: Arc<Mutex<LR35902CPU>>,
    screen_buffer: [u8; RESX * RESY * 4],
    screen_texture: egui::TextureHandle,
    debugger: debugger::DebuggerState,
    video_channel_rc: Receiver<Vbuf>,
}

impl XenoGBUI {
    pub fn new(
        ctx: &eframe::CreationContext<'_>,
        video_channel_rc: Receiver<Vbuf>,
        cpu: Arc<Mutex<LR35902CPU>>,
        debug: bool,
    ) -> Self {
        let screen_buffer = [0xff; RESX * RESY * 4];
        let screen_texture = ctx.egui_ctx.load_texture(
            "screen",
            egui::ColorImage::from_rgba_unmultiplied([RESX, RESY], &screen_buffer),
            egui::TextureOptions::NEAREST,
        );

        let debugger = debugger::DebuggerState::new(ctx, debug, cpu.clone());

        Self {
            cpu,
            screen_buffer,
            screen_texture,
            debugger,
            video_channel_rc,
        }
    }

    fn render_vbuf(&mut self) {
        let vbuf = self
            .video_channel_rc
            .recv()
            .expect("Could not receive video buffer");

        for i in 0..vbuf.len() {
            self.screen_buffer[i * 4] = vbuf[i];
            self.screen_buffer[i * 4 + 1] = vbuf[i];
            self.screen_buffer[i * 4 + 2] = vbuf[i];
        }
    }
}

impl eframe::App for XenoGBUI {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        ctx.input(|inp| {
            let mut cpu = self.cpu.lock().unwrap();
            for (emu_key, ui_key) in KEYMAP.entries() {
                if inp.key_pressed(*ui_key) {
                    cpu.bus.io.joypad.press(*emu_key);
                }
                if inp.key_released(*ui_key) {
                    cpu.bus.io.joypad.release(*emu_key);
                }
            }

            if inp.modifiers.ctrl && inp.key_released(DEBUGGER_KEY) {
                self.debugger.enabled = !self.debugger.enabled;
            }
        });

        self.render_vbuf();
        ctx.tex_manager().write().set(
            self.screen_texture.id(),
            egui::epaint::ImageDelta::full(
                egui::ColorImage::from_rgba_unmultiplied([RESX, RESY], &self.screen_buffer),
                egui::TextureOptions::NEAREST,
            ),
        );

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
