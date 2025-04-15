use super::ui::{XenoGBUI, WINDOW_SIZE};

use eframe::egui;
use egui::{
    include_image, CentralPanel, Color32, Context, CornerRadius, FontId, Frame, Hyperlink, Image,
    Margin, RichText, ScrollArea,
};

impl XenoGBUI {
    pub fn bsod_ui(&self, ctx: &Context) {
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
                            ui.add(Image::new(include_image!("../assets/bsod.png")).fit_to_original_size(scale * 0.35));
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
