use super::debugger::DebuggerUi;
use crate::debugger::{DebuggerCommand, EmuSnapshot};
use crate::io::joypad::JOYPAD_INPUT;
use crate::io::video::ppu::{Vbuf, RESX, RESY};
use crate::io_event::IOEvent;
use crate::mem::bus::Bus;
use crate::run_emu::{run_emu_thread, EmuState};
use cphf::{phf_ordered_map, OrderedMap};
use crossbeam_channel::{Receiver, Sender};
use eframe::egui;
use egui::{
    include_image, CentralPanel, Color32, Context, CornerRadius, FontId, Frame, Hyperlink, Image,
    Key, Margin, RichText, ScrollArea, ViewportBuilder,
};
use egui_extras::install_image_loaders;

use std::path::PathBuf;

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

struct XenoGBUI {
    screen_buffer: [u8; RESX * RESY * 4],
    screen_texture: egui::TextureHandle,
    debugger: DebuggerUi,
    video_channel_rc: Receiver<Vbuf>,
    events_sd: Sender<IOEvent>,
    dbg_commands_sd: Sender<DebuggerCommand>,
    emu_state: EmuState,
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
            egui::ColorImage::from_rgba_unmultiplied([RESX, RESY], &screen_buffer),
            egui::TextureOptions::NEAREST,
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
            emu_state,
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

    fn bsod(&self, ctx: &Context) {
        let crash = self.emu_state.crash_info();

        CentralPanel::default()
            .frame(Frame::new().fill(Color32::from_rgb(0, 0x78, 0xd7)).inner_margin(Margin::from(20)))
            .show(ctx, |ui| {
                ui.style_mut().visuals.override_text_color = Some(Color32::WHITE);
                let scale = ui.min_rect().width() / WINDOW_SIZE[0];

                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(":(")
                            .size(72.0 * scale)
                            .font(FontId::proportional(72.0 * scale)),
                    );

                    ui.add_space(30.0 * scale);

                    ui.label(
                        RichText::new("Your emulator ran into a problem and needs to restart.")
                            .size(22.0 * scale)
                    );

                    ui.add_space(40.0 * scale);

                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.add(Image::new(include_image!("assets/bsod.png")).fit_to_original_size(scale * 0.35));
                            ui.add_space(200.0);
                        });

                        ui.vertical(|ui| {
                            ui.vertical(|ui| {
                                ui.spacing_mut().item_spacing.y = 0.0;
                                ui.label(
                                    RichText::new("For more information about this issue and possible fixes, visit ")
                                        .size(14.0 * scale)
                                );
                                ui.add(
                                    Hyperlink::from_label_and_url(
                                        RichText::new("https://github.com/Sacrimento/xenogb")
                                            .size(14.0 * scale),
                                        "https://github.com/Sacrimento/xenogb"
                                    )
                                );
                            });

                            ui.add_space(10.0 * scale);

                            ui.label(
                                RichText::new(format!("If you call a support person (please don't), give them this info:\n"))
                                    .size(14.0 * scale)
                            );

                            ui.collapsing(
                                RichText::new(format!("Panicked: {}", crash.reason))
                                    .size(14.0 * scale),
                                |ui| {
                                    ScrollArea::vertical()
                                        .show(ui, |ui| {
                                            Frame::new()
                                                .fill(Color32::from_rgb(0, 0x5b, 0xa5))
                                                .corner_radius(CornerRadius::same(6))
                                                .show(ui, |ui| {
                                                    let bt = RichText::new(crash.backtrace.clone())
                                                        .monospace()
                                                        .size(14.0 * scale);

                                                    ui.label(bt);
                                                });
                                        });
                                },
                            );
                        });
                     });
                });

            });
    }
}

impl eframe::App for XenoGBUI {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        if self.emu_state.is_dead() {
            return self.bsod(ctx);
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
