use crate::io::joypad::JOYPAD_INPUT;
use crate::io::video::ppu::VIDEO_BUFFER;
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

const GB_RES_X: usize = 160;
const GB_RES_Y: usize = 144;

const SCALE: usize = 4;

const WINDOW_SIZE: [f32; 2] = [(GB_RES_X * SCALE) as f32, (GB_RES_Y * SCALE) as f32];

const COLORS: [u8; 4] = [0xff, 0xaa, 0x55, 0x00];

pub struct XenoGBUI {
    cpu: Arc<Mutex<LR35902CPU>>,
    // vpu_buffer: Vec<u8>,
    // vpu_texture: egui::TextureHandle,
}

impl XenoGBUI {
    pub fn new(ctx: &eframe::CreationContext<'_>, cpu: Arc<Mutex<LR35902CPU>>) -> Self {
        // let vpu_buffer = vec![0xff; GB_RES_X * GB_RES_Y * 4 * SCALE * SCALE];
        // let vpu_texture = ctx.egui_ctx.load_texture(
        //     "screen",
        //     egui::ColorImage::from_rgba_unmultiplied(
        //         [GB_RES_X * SCALE, GB_RES_Y * SCALE],
        //         &vpu_buffer,
        //     ),
        //     egui::TextureOptions::NEAREST,
        // );

        ctx.egui_ctx.set_zoom_factor(2.5);

        Self {
            cpu,
            // vpu_buffer,
            // vpu_texture,
        }
    }

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
                    egui::Color32::from_gray(COLORS[(hi | lo) as usize]),
                ));
            }
        }
        ret_vec
    }

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

    fn render_vbuf(&mut self, ui: &mut egui::Ui) {
        let vbuf = VIDEO_BUFFER.lock().unwrap();

        for y in 0..GB_RES_Y {
            for x in 0..GB_RES_X {
                let rect = egui::Rect::from_min_size(
                    egui::pos2(x as f32, y as f32),
                    vec2(SCALE as f32, SCALE as f32),
                );
                ui.painter().rect_filled(
                    rect,
                    egui::CornerRadius::ZERO,
                    egui::Color32::from_gray((vbuf[y * GB_RES_X + x]) as u8),
                );
            }
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

        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_vbuf(ui);
            // self.render_vram(ui);
        });

        ctx.request_repaint();
    }
}
