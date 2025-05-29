use std::fmt::Display;

use eframe::egui;
use egui::{
    widgets::color_picker::{color_picker_color32, Alpha},
    Color32, ComboBox, Slider, TopBottomPanel, Ui,
};

#[allow(nonstandard_style)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphicsMode {
    NORMAL,
    TINT_TO_WHITE(Color32),
    TINT_TO_BLACK(Color32),
    RAINBOW,
}

impl Display for GraphicsMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphicsMode::NORMAL => write!(f, "Normal"),
            GraphicsMode::TINT_TO_WHITE(_) => write!(f, "Tinted whites"),
            GraphicsMode::TINT_TO_BLACK(_) => write!(f, "Tinted blacks"),
            GraphicsMode::RAINBOW => write!(f, "Rainbow"),
        }
    }
}

pub struct Settings {
    // Sound
    sound_volume: f32,
    sound_mute: bool,

    // Graphics
    pub graphics_mode: GraphicsMode,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            sound_volume: 0.5,
            sound_mute: false,
            graphics_mode: GraphicsMode::NORMAL,
        }
    }
}

impl Settings {
    pub fn new() -> Self {
        Settings::from_file().unwrap_or_default()
    }

    fn from_file() -> Result<Self, ()> {
        // Load settings from a file
        Err(())
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        TopBottomPanel::top("settings").show(ui.ctx(), |ui| {
            ui.label("Settings");

            // Sound settings
            ui.horizontal(|ui| {
                ui.label("Sound Volume:");
                ui.add(Slider::new(&mut self.sound_volume, 0.0..=1.0).text("Volume"));
            });

            ui.checkbox(&mut self.sound_mute, "Mute Sound");

            // Graphics settings
            ui.label("Graphics Mode:");
            ui.horizontal(|ui| {
                ComboBox::from_label("Graphics modifiers")
                    .selected_text(self.graphics_mode.to_string())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.graphics_mode,
                            GraphicsMode::NORMAL,
                            GraphicsMode::NORMAL.to_string(),
                        );
                        ui.selectable_value(
                            &mut self.graphics_mode,
                            GraphicsMode::TINT_TO_WHITE(Color32::WHITE),
                            GraphicsMode::TINT_TO_WHITE(Color32::WHITE).to_string(),
                        );

                        ui.selectable_value(
                            &mut self.graphics_mode,
                            GraphicsMode::TINT_TO_BLACK(Color32::BLACK),
                            GraphicsMode::TINT_TO_BLACK(Color32::BLACK).to_string(),
                        );
                        ui.selectable_value(
                            &mut self.graphics_mode,
                            GraphicsMode::RAINBOW,
                            GraphicsMode::RAINBOW.to_string(),
                        );
                    });

                match self.graphics_mode {
                    GraphicsMode::TINT_TO_BLACK(c) => {
                        let mut c = c;
                        color_picker_color32(ui, &mut c, Alpha::Opaque);
                        self.graphics_mode = GraphicsMode::TINT_TO_BLACK(c);
                    }
                    GraphicsMode::TINT_TO_WHITE(c) => {
                        let mut c = c;
                        color_picker_color32(ui, &mut c, Alpha::Opaque);
                        self.graphics_mode = GraphicsMode::TINT_TO_WHITE(c);
                    }
                    _ => (),
                }
            });
        });
    }
}
