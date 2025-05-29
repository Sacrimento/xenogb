use crate::core::io::joypad::JOYPAD_INPUT;
use crate::core::io::video::ppu::{Vbuf, RESX, RESY};
use crate::core::io_event::IOEvent;
use crate::core::run_emu::EmuState;
use crate::debugger::{DebuggerCommand, EmuSnapshot};
use crate::ui::{
    debugger::DebuggerUi,
    main::settings::{GraphicsMode, Settings},
};

use cphf::{phf_ordered_map, OrderedMap};
use crossbeam_channel::{Receiver, Sender};
use eframe::egui;
use egui::{
    epaint, CentralPanel, Color32, ColorImage, Context, Frame, Image, Key, Pos2, Rect, Sense,
    TextureOptions, Vec2,
};
use egui_extras::install_image_loaders;

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

pub struct XenoGBUI {
    screen_buffer: [u8; RESX * RESY * 4],
    screen_texture: egui::TextureHandle,

    debugger: DebuggerUi,

    video_channel_rc: Receiver<Vbuf>,
    events_sd: Sender<IOEvent>,
    dbg_commands_sd: Sender<DebuggerCommand>,

    settings: Settings,

    frame: u64,

    pub emu_state: EmuState,
}

impl XenoGBUI {
    pub fn new(
        ctx: &eframe::CreationContext<'_>,
        emu_state: EmuState,
        events_sd: Sender<IOEvent>,
        video_channel_rc: Receiver<Vbuf>,
        dbg_commands_sd: Sender<DebuggerCommand>,
        dbg_data_rc: Receiver<EmuSnapshot>,
        debug: bool,
    ) -> Self {
        let screen_buffer = [0xff; RESX * RESY * 4];
        let screen_texture = ctx.egui_ctx.load_texture(
            "screen",
            ColorImage::from_rgba_unmultiplied([RESX, RESY], &screen_buffer),
            TextureOptions::NEAREST,
        );

        let debugger = DebuggerUi::new(ctx, debug, dbg_commands_sd.clone(), dbg_data_rc);

        install_image_loaders(&ctx.egui_ctx);

        Self {
            screen_buffer,
            screen_texture,
            debugger,
            video_channel_rc,
            events_sd,
            dbg_commands_sd,
            settings: Settings::new(),
            emu_state,
            frame: 0,
        }
    }

    fn render_vbuf(&mut self, ctx: &Context) {
        if let Ok(vbuf) = self.video_channel_rc.try_recv() {
            for (i, pixel) in vbuf.iter().enumerate() {
                self.screen_buffer[i * 4] = *pixel;
                self.screen_buffer[i * 4 + 1] = *pixel;
                self.screen_buffer[i * 4 + 2] = *pixel;
            }

            ctx.tex_manager().write().set(
                self.screen_texture.id(),
                epaint::ImageDelta::full(
                    ColorImage::from_rgba_unmultiplied([RESX, RESY], &self.screen_buffer),
                    TextureOptions::NEAREST,
                ),
            );
        }
    }
}

impl eframe::App for XenoGBUI {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        if self.debugger.enabled {
            self.debugger.ui(ctx);
        }

        if self.emu_state.is_dead() {
            return self.bsod_ui(ctx);
        }

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

        CentralPanel::default().frame(Frame::NONE).show(ctx, |ui| {
            let mut screen = Image::from_texture(&self.screen_texture)
                .fit_to_original_size(SCALE as f32)
                .maintain_aspect_ratio(true)
                .sense(Sense::hover());

            match self.settings.graphics_mode {
                GraphicsMode::NORMAL => (),
                GraphicsMode::TINT(c) => {
                    screen = screen.tint(c);
                }
                GraphicsMode::RAINBOW => {
                    // let dir: f32 = if self.frame / 64 % 2 == 0 { 1.0 } else { -1.0 };
                    // screen =
                    //     screen.rotate(self.frame as f32 % 64.0 * dir * 0.005, Vec2::splat(0.5));

                    // screen = screen.tint(Color32::from_rgb(
                    //     (self.frame % 256) as u8,
                    //     ((self.frame + 85) % 256) as u8,
                    //     ((self.frame + 170) % 256) as u8,
                    // ));
                }
            }

            let res = ui.add(screen);

            if res.ctx.pointer_latest_pos().is_some_and(|pos| {
                Rect::from_min_max(
                    Pos2::ZERO,
                    Pos2::new(res.rect.width(), res.rect.height() / 2.0),
                )
                .contains(pos)
            }) {
                self.settings.ui(ui);
            }
        });

        ctx.request_repaint();

        self.frame += 1;
    }
}
