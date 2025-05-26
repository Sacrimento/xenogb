use crate::core::io::video::lcd::LCD;
use crate::debugger::EmuSnapshot;
use crossbeam_channel::Receiver;
use eframe::egui::{
    self, epaint, vec2, Align, Color32, ColorImage, CornerRadius, Image, Layout, Rect, Scene,
    Sense, Stroke, StrokeKind, TextureHandle, TextureOptions, Ui,
};

const VRAM_SCALE: usize = 5;
const VRAMX: usize = 16;
const VRAMY: usize = 24;

pub struct PpuUi {
    vram_textures: [TextureHandle; VRAMX * VRAMY],

    selected_tile_idx: Option<usize>,
    selected_tile_scene_rect: Rect,

    dbg_data_rc: Receiver<EmuSnapshot>,
}

impl PpuUi {
    pub fn new(ctx: &eframe::CreationContext<'_>, dbg_data_rc: Receiver<EmuSnapshot>) -> Self {
        let vram_textures = [(); VRAMX * VRAMY].map(|()| {
            ctx.egui_ctx.load_texture(
                "vram",
                ColorImage::from_gray([8, 8], &[0xff; 8 * 8]),
                TextureOptions::NEAREST,
            )
        });

        Self {
            vram_textures,
            selected_tile_idx: None,
            selected_tile_scene_rect: Rect::ZERO,
            dbg_data_rc,
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        ui.with_layout(Layout::left_to_right(Align::TOP), |ui| {
            self.vram_grid_ui(ui);
            ui.separator();
            self.selected_tile_ui(ui);
        });
    }

    fn vram_grid_ui(&mut self, ui: &mut Ui) {
        egui::ScrollArea::vertical()
            .drag_to_scroll(false)
            .show(ui, |ui| {
                egui::Grid::new("vram-grid")
                    .spacing(vec2(0., 0.))
                    .show(ui, |ui| {
                        for y in 0..VRAMY {
                            for x in 0..VRAMX {
                                self.tile_ui(ui, x, y);
                            }
                            ui.end_row();
                        }
                    });
            });
    }

    fn render_tile(&self, ui: &mut Ui, tile_idx: usize) {
        let Some(vram) = self
            .dbg_data_rc
            .try_iter()
            .last()
            .and_then(|d| Some(d.vram))
        else {
            return;
        };

        let mut tile_buffer: [u8; 64] = [0; 64];
        let mut tile_buffer_idx: usize = 0;

        for tile_y in (0..16).step_by(2) {
            let b1 = vram[(tile_idx * 16) + tile_y];
            let b2 = vram[(tile_idx * 16) + tile_y + 1];

            for bit in (0..8).rev().step_by(1) {
                let hi = ((b2 >> bit) & 1) << 1;
                let lo = (b1 >> bit) & 1;

                tile_buffer[tile_buffer_idx] = LCD::get_pixel(hi | lo);
                tile_buffer_idx += 1;
            }
        }

        ui.ctx().tex_manager().write().set(
            self.vram_textures[tile_idx].id(),
            epaint::ImageDelta::full(
                ColorImage::from_gray([8, 8], &tile_buffer),
                TextureOptions::NEAREST,
            ),
        );
    }

    fn tile_ui(&mut self, ui: &mut Ui, x: usize, y: usize) {
        let tile_idx = y * VRAMX + x;
        self.render_tile(ui, tile_idx);

        let img = Image::from_texture(&self.vram_textures[tile_idx])
            .fit_to_original_size(VRAM_SCALE as f32)
            .sense(Sense::click());
        let resp = ui.add(img);
        let tile_rect = resp.rect;

        if resp.clicked() {
            self.tile_clicked(tile_idx);
        }

        if resp.hovered() {
            ui.painter().rect_stroke(
                tile_rect,
                CornerRadius::ZERO,
                Stroke::new(VRAM_SCALE as f32, Color32::LIGHT_RED),
                StrokeKind::Inside,
            );
        }

        if let Some(selected_idx) = self.selected_tile_idx {
            if tile_idx == selected_idx {
                ui.painter().rect_stroke(
                    tile_rect,
                    CornerRadius::ZERO,
                    Stroke::new(VRAM_SCALE as f32, Color32::RED),
                    StrokeKind::Inside,
                );
            }
        }
    }

    fn tile_clicked(&mut self, tile_idx: usize) {
        match self.selected_tile_idx {
            Some(idx) => {
                if idx == tile_idx {
                    self.selected_tile_idx = None
                } else {
                    self.selected_tile_idx = Some(tile_idx)
                }
            }
            None => self.selected_tile_idx = Some(tile_idx),
        };
    }

    fn selected_tile_ui(&mut self, ui: &mut Ui) {
        let scene = Scene::new()
            .max_inner_size([350.0, 1000.0])
            .zoom_range(0.1..=2.0);

        scene.show(ui, &mut self.selected_tile_scene_rect, |ui| {
            if let Some(selected_tile_idx) = self.selected_tile_idx {
                ui.add(
                    Image::from_texture(&self.vram_textures[selected_tile_idx])
                        .fit_to_original_size((VRAM_SCALE * 4) as f32),
                );
            }
        });
    }
}
