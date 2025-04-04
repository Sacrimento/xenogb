use crate::io::joypad::JOYPAD_INPUT;
use crate::io::video::lcd::LCD;
use crate::io::video::ppu::{RESX, RESY, VIDEO_BUFFER};
use crate::LR35902CPU;
use cphf::{phf_ordered_map, OrderedMap};
use eframe::egui;
use eframe::{self, egui::vec2};
use std::sync::{Arc, Mutex};

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

const SCALE: usize = 4;

pub const WINDOW_SIZE: [f32; 2] = [(RESX * SCALE) as f32, (RESY * SCALE) as f32];

pub struct XenoGBUI {
    cpu: Arc<Mutex<LR35902CPU>>,
    screen_buffer: [u8; RESX * RESY * 4],
    screen_texture: egui::TextureHandle,
}

impl XenoGBUI {
    pub fn new(ctx: &eframe::CreationContext<'_>, cpu: Arc<Mutex<LR35902CPU>>) -> Self {
        let screen_buffer = [0xff; RESX * RESY * 4];
        let screen_texture = ctx.egui_ctx.load_texture(
            "screen",
            egui::ColorImage::from_rgba_unmultiplied([RESX, RESY], &screen_buffer),
            egui::TextureOptions::NEAREST,
        );

        ctx.egui_ctx.set_zoom_factor(SCALE as f32);

        Self {
            cpu,
            screen_buffer,
            screen_texture,
        }
    }

    #[allow(dead_code)]
    fn render_tile(&mut self, tile_num: u32, x: u32, y: u32) -> Vec<(egui::Rect, egui::Color32)> {
        let cpu = self.cpu.lock().unwrap();
        let mut ret_vec: Vec<(egui::Rect, egui::Color32)> = vec![];

        for tile_y in (0..16).step_by(2) {
            let b1 = cpu.bus.read((0x8000 + (tile_num * 16) + tile_y) as u16);
            let b2 = cpu.bus.read((0x8000 + (tile_num * 16) + tile_y + 1) as u16);

            for bit in (0..8).rev().step_by(1) {
                let hi = ((b2 >> bit) & 1) << 1;
                let lo = (b1 >> bit) & 1;

                ret_vec.push((
                    egui::Rect::from_min_size(
                        egui::pos2((x + (7 - bit)) as f32, (tile_y as u32 / 2 + y) as f32),
                        vec2(SCALE as f32, SCALE as f32),
                    ),
                    egui::Color32::from_gray(LCD::get_pixel(hi | lo)),
                ));
            }
        }
        ret_vec
    }

    #[allow(dead_code)]
    fn render_vram(&mut self, ui: &mut egui::Ui) {
        let mut tile_num = 0;
        let mut x_render = 0;
        let mut y_render = 0;

        for y in 0..24 {
            for x in 0..16 {
                for (rect, color) in
                    self.render_tile(tile_num, (x_render + x) as u32, (y_render + y) as u32)
                {
                    ui.painter()
                        .rect_filled(rect, egui::CornerRadius::ZERO, color);
                }
                x_render += 8 as u32;
                tile_num += 1;
            }
            y_render += 8 as u32;
            x_render = 0;
        }
    }

    fn render_vbuf(&mut self) {
        let vbuf = VIDEO_BUFFER.lock().unwrap();

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
                ui.image(&self.screen_texture);
            });

        ctx.request_repaint();
    }
}
