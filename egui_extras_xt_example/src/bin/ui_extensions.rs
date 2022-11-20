use eframe::egui::{self, DragValue, Response};
use eframe::emath::vec2;

use strum::{Display, EnumIter, IntoEnumIterator};

use egui_extras_xt::ui::optional_value_widget::OptionalValueWidget;
use egui_extras_xt::ui::widgets_from::{WidgetsFromIterator, WidgetsFromRange, WidgetsFromSlice};

#[derive(EnumIter, Clone, Copy, PartialEq, Display)]
enum SevenSecretWeapons {
    #[strum(to_string = "Missile")]
    Missile,

    #[strum(to_string = "Metal detector")]
    MetalDetector,

    #[strum(to_string = "Fishing pole")]
    FishingPole,

    #[strum(to_string = "Mr. Analysis")]
    MrAnalysis,

    #[strum(to_string = "Magnet")]
    Magnet,

    #[strum(to_string = "Bug sweeper")]
    BugSweeper,
}

struct UiExtensionsExample {
    optional_usize: Option<usize>,
    secret_weapon: SevenSecretWeapons,
    some_value: usize,
}

impl Default for UiExtensionsExample {
    fn default() -> Self {
        Self {
            optional_usize: Some(1234),
            secret_weapon: SevenSecretWeapons::MetalDetector,
            some_value: 1,
        }
    }
}

impl eframe::App for UiExtensionsExample {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        fn debug_print_response(widget_name: &'static str, response: Response) {
            if response.changed() {
                println!("{widget_name:} changed");
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.push_id("optional_value", |ui| {
                ui.group(|ui| {
                    debug_print_response(
                        "optional_value_widget",
                        ui.optional_value_widget(&mut self.optional_usize, |ui, value| {
                            ui.add(DragValue::new(value))
                        }),
                    );
                });
            });
            ui.add_space(8.0);

            ui.push_id("from_iter", |ui| {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        debug_print_response(
                            "selectable_value_from_iter",
                            ui.selectable_value_from_iter(
                                &mut self.secret_weapon,
                                SevenSecretWeapons::iter(),
                            ),
                        );
                    });

                    ui.horizontal(|ui| {
                        debug_print_response(
                            "combobox_from_iter",
                            ui.combobox_from_iter(
                                "Secret weapon",
                                &mut self.secret_weapon,
                                SevenSecretWeapons::iter(),
                            ),
                        );
                    });

                    ui.horizontal(|ui| {
                        debug_print_response(
                            "radio_value_from_iter",
                            ui.radio_value_from_iter(
                                &mut self.secret_weapon,
                                SevenSecretWeapons::iter(),
                            ),
                        );
                    });
                });
            });
            ui.add_space(8.0);

            ui.push_id("from_slice", |ui| {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        debug_print_response(
                            "selectable_value_from_slice",
                            ui.selectable_value_from_slice(
                                &mut self.secret_weapon,
                                &[
                                    SevenSecretWeapons::Missile,
                                    SevenSecretWeapons::MetalDetector,
                                    SevenSecretWeapons::FishingPole,
                                    SevenSecretWeapons::MrAnalysis,
                                    SevenSecretWeapons::Magnet,
                                    SevenSecretWeapons::BugSweeper,
                                ],
                            ),
                        );
                    });

                    ui.horizontal(|ui| {
                        debug_print_response(
                            "combobox_from_slice",
                            ui.combobox_from_slice(
                                "Secret weapon",
                                &mut self.secret_weapon,
                                &[
                                    SevenSecretWeapons::Missile,
                                    SevenSecretWeapons::MetalDetector,
                                    SevenSecretWeapons::FishingPole,
                                    SevenSecretWeapons::MrAnalysis,
                                    SevenSecretWeapons::Magnet,
                                    SevenSecretWeapons::BugSweeper,
                                ],
                            ),
                        );
                    });

                    ui.horizontal(|ui| {
                        debug_print_response(
                            "radio_value_from_slice",
                            ui.radio_value_from_slice(
                                &mut self.secret_weapon,
                                &[
                                    SevenSecretWeapons::Missile,
                                    SevenSecretWeapons::MetalDetector,
                                    SevenSecretWeapons::FishingPole,
                                    SevenSecretWeapons::MrAnalysis,
                                    SevenSecretWeapons::Magnet,
                                    SevenSecretWeapons::BugSweeper,
                                ],
                            ),
                        );
                    });
                });
            });
            ui.add_space(8.0);

            ui.push_id("from_range", |ui| {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        debug_print_response(
                            "selectable_value_from_range",
                            ui.selectable_value_from_range(&mut self.some_value, 1..=7),
                        );
                    });

                    ui.horizontal(|ui| {
                        debug_print_response(
                            "combobox_from_range",
                            ui.combobox_from_range("Some value", &mut self.some_value, 1..=7),
                        );
                    });

                    ui.horizontal(|ui| {
                        debug_print_response(
                            "radio_value_from_range",
                            ui.radio_value_from_range(&mut self.some_value, 1..=7),
                        );
                    });
                });
            });
            ui.add_space(8.0);
        });
    }
}

fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(vec2(640.0, 480.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Ui extensions",
        options,
        Box::new(|_| Box::new(UiExtensionsExample::default())),
    );
}
