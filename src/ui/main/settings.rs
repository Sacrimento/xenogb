use std::fmt::Display;

use crossbeam_channel::Sender;
use eframe::egui::{self, CornerRadius};
use egui::{
    widgets::color_picker::{color_picker_color32, Alpha},
    Color32, ComboBox, Key, Order, Slider, TopBottomPanel, Ui, Window,
};
use egui_extras::{Column, TableBuilder, TableRow};
use indexmap::IndexMap;
use itertools::Itertools;

use crate::core::{io::joypad::JOYPAD_INPUT, io_event::IOEvent};

#[allow(nonstandard_style)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphicsSettings {
    #[default]
    NORMAL,
    TINT_TO_WHITE(Color32),
    TINT_TO_BLACK(Color32),
    RAINBOW,
}

impl Display for GraphicsSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphicsSettings::NORMAL => write!(f, "Normal"),
            GraphicsSettings::TINT_TO_WHITE(_) => write!(f, "Tinted whites"),
            GraphicsSettings::TINT_TO_BLACK(_) => write!(f, "Tinted blacks"),
            GraphicsSettings::RAINBOW => write!(f, "Rainbow"),
        }
    }
}

impl GraphicsSettings {
    pub fn ui(&mut self, ui: &mut Ui) {
        ui.label("Graphics Mode:");
        ui.horizontal(|ui| {
            ComboBox::from_label("Graphics modifiers")
                .selected_text(self.to_string())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        self,
                        GraphicsSettings::NORMAL,
                        GraphicsSettings::NORMAL.to_string(),
                    );
                    ui.selectable_value(
                        self,
                        GraphicsSettings::TINT_TO_WHITE(Color32::WHITE),
                        GraphicsSettings::TINT_TO_WHITE(Color32::WHITE).to_string(),
                    );

                    ui.selectable_value(
                        self,
                        GraphicsSettings::TINT_TO_BLACK(Color32::BLACK),
                        GraphicsSettings::TINT_TO_BLACK(Color32::BLACK).to_string(),
                    );
                    ui.selectable_value(
                        self,
                        GraphicsSettings::RAINBOW,
                        GraphicsSettings::RAINBOW.to_string(),
                    );
                });

            match self {
                GraphicsSettings::TINT_TO_BLACK(mut c) => {
                    color_picker_color32(ui, &mut c, Alpha::Opaque);
                    *self = GraphicsSettings::TINT_TO_BLACK(c);
                }
                GraphicsSettings::TINT_TO_WHITE(mut c) => {
                    color_picker_color32(ui, &mut c, Alpha::Opaque);
                    *self = GraphicsSettings::TINT_TO_WHITE(c);
                }
                _ => (),
            }
        });
    }
}

struct SoundSettings {
    volume: f32,
    mute: bool,
}

impl Default for SoundSettings {
    fn default() -> Self {
        Self {
            volume: 0.5,
            mute: false,
        }
    }
}

impl SoundSettings {
    pub fn ui(&mut self, ui: &mut Ui, sender: &Sender<IOEvent>) {
        ui.horizontal(|ui| {
            ui.label("Sound Volume:");
            let res = ui.add(Slider::new(&mut self.volume, 0.0..=1.0).text("Volume"));
            if res.lost_focus() || res.dragged() {
                sender
                    .send(IOEvent::SOUND_VOLUME(self.volume))
                    .expect("Could not send io event")
            }
        });

        if ui.checkbox(&mut self.mute, "Mute Sound").changed() {
            sender
                .send(IOEvent::SOUND_MUTE(self.mute))
                .expect("Could not send io event");
        }
    }
}

pub struct KeymapSettings {
    pub map: IndexMap<u8, Key>,
    modal_open: bool,
    modal_binder_key: u8,
}

impl Default for KeymapSettings {
    fn default() -> Self {
        let mut map = IndexMap::new();
        map.insert(JOYPAD_INPUT::A, Key::A);
        map.insert(JOYPAD_INPUT::B, Key::S);
        map.insert(JOYPAD_INPUT::UP, Key::ArrowUp);
        map.insert(JOYPAD_INPUT::DOWN, Key::ArrowDown);
        map.insert(JOYPAD_INPUT::LEFT, Key::ArrowLeft);
        map.insert(JOYPAD_INPUT::RIGHT, Key::ArrowRight);
        map.insert(JOYPAD_INPUT::SELECT, Key::Space);
        map.insert(JOYPAD_INPUT::START, Key::Enter);

        Self {
            map,
            modal_open: false,
            modal_binder_key: 0,
        }
    }
}

impl KeymapSettings {
    pub fn _new() -> Self {
        todo!()
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        if ui.button("Keybinds").clicked() {
            self.modal_open = !self.modal_open;
        }

        let _duplicates: Vec<&Key> = self.map.values().duplicates().collect();

        Window::new("Keybinds")
            .open(&mut self.modal_open)
            .vscroll(false)
            .resizable(false)
            .collapsible(false)
            .show(ui.ctx(), |ui| {
                TableBuilder::new(ui)
                    .column(Column::auto().resizable(false))
                    .column(Column::remainder())
                    .body(|mut body| {
                        for (emu_key, ui_key) in self.map.iter_mut() {
                            body.row(30.0, |mut row| {
                                row.col(|ui| {
                                    ui.label(JOYPAD_INPUT::display(*emu_key));
                                });
                                row.col(|ui| {
                                    if ui.button(ui_key.name().to_string()).clicked() {
                                        self.modal_binder_key = *emu_key;
                                    }

                                    if self.modal_binder_key == *emu_key {
                                        Window::new("modal-bind")
                                            .title_bar(false)
                                            .resizable(false)
                                            .collapsible(false)
                                            .movable(false)
                                            .order(Order::Foreground)
                                            .show(ui.ctx(), |ui| {
                                                ui.label(format!(
                                                "Press any key to bind to {}. Press ESC to cancel",
                                                JOYPAD_INPUT::display(*emu_key)
                                            ));

                                                ui.ctx().input(|inp| {
                                                    if let Some(key_pressed) =
                                                        inp.keys_down.iter().next()
                                                    {
                                                        if *key_pressed != Key::Escape {
                                                            *ui_key = *key_pressed;
                                                        }

                                                        self.modal_binder_key = 0;
                                                    }
                                                });
                                            });
                                    }
                                });
                            });
                        }
                    });
            });
    }
}

pub struct Settings {
    sound: SoundSettings,
    pub graphics: GraphicsSettings,
    pub keymap: KeymapSettings,

    io_event_sd: Sender<IOEvent>,
}

impl Settings {
    pub fn new(sender: Sender<IOEvent>) -> Self {
        let sound_settings = SoundSettings::default();
        // Synchronize emulation's volume with settings volume
        _ = sender.send(IOEvent::SOUND_VOLUME(sound_settings.volume));
        Settings {
            sound: sound_settings,
            graphics: GraphicsSettings::default(),
            keymap: KeymapSettings::default(),
            io_event_sd: sender,
        }
    }

    fn _from_file(_sender: Sender<IOEvent>) -> Result<Self, ()> {
        todo!()
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        TopBottomPanel::top("settings").show(ui.ctx(), |ui| {
            ui.label("Settings");

            self.sound.ui(ui, &self.io_event_sd);
            self.graphics.ui(ui);
            self.keymap.ui(ui);
        });
    }
}
