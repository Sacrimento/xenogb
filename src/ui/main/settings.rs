use std::fmt::Display;

use crossbeam_channel::Sender;
use eframe::egui;
use egui::{
    widgets::color_picker::{color_picker_color32, Alpha},
    Color32, ComboBox, Slider, TopBottomPanel, Ui,
};

use crate::core::io_event::IOEvent;

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

pub struct Settings {
    sound: SoundSettings,
    pub graphics: GraphicsSettings,

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
            io_event_sd: sender,
        }
    }

    fn from_file(sender: Sender<IOEvent>) -> Result<Self, ()> {
        todo!()
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        TopBottomPanel::top("settings").show(ui.ctx(), |ui| {
            ui.label("Settings");

            // Sound settings
            ui.horizontal(|ui| {
                ui.label("Sound Volume:");
                let res = ui.add(Slider::new(&mut self.sound.volume, 0.0..=1.0).text("Volume"));
                if res.lost_focus() || res.dragged() {
                    self.io_event_sd
                        .send(IOEvent::SOUND_VOLUME(self.sound.volume))
                        .expect("Could not send io event")
                }
            });

            let res = ui.checkbox(&mut self.sound.mute, "Mute Sound");
            if res.changed() {
                self.io_event_sd
                    .send(IOEvent::SOUND_MUTE(self.sound.mute))
                    .expect("Could not send io event");
            }

            // Graphics settings
            ui.label("Graphics Mode:");
            ui.horizontal(|ui| {
                ComboBox::from_label("Graphics modifiers")
                    .selected_text(self.graphics.to_string())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.graphics,
                            GraphicsSettings::NORMAL,
                            GraphicsSettings::NORMAL.to_string(),
                        );
                        ui.selectable_value(
                            &mut self.graphics,
                            GraphicsSettings::TINT_TO_WHITE(Color32::WHITE),
                            GraphicsSettings::TINT_TO_WHITE(Color32::WHITE).to_string(),
                        );

                        ui.selectable_value(
                            &mut self.graphics,
                            GraphicsSettings::TINT_TO_BLACK(Color32::BLACK),
                            GraphicsSettings::TINT_TO_BLACK(Color32::BLACK).to_string(),
                        );
                        ui.selectable_value(
                            &mut self.graphics,
                            GraphicsSettings::RAINBOW,
                            GraphicsSettings::RAINBOW.to_string(),
                        );
                    });

                match self.graphics {
                    GraphicsSettings::TINT_TO_BLACK(c) => {
                        let mut c = c;
                        color_picker_color32(ui, &mut c, Alpha::Opaque);
                        self.graphics = GraphicsSettings::TINT_TO_BLACK(c);
                    }
                    GraphicsSettings::TINT_TO_WHITE(c) => {
                        let mut c = c;
                        color_picker_color32(ui, &mut c, Alpha::Opaque);
                        self.graphics = GraphicsSettings::TINT_TO_WHITE(c);
                    }
                    _ => (),
                }
            });
        });
    }
}
