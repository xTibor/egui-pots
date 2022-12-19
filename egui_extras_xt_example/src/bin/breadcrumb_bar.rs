use std::path::PathBuf;

use eframe::egui;
use eframe::emath::vec2;
use egui_extras_xt::filesystem::breadcrumb_bar::breadcrumb_bar;

struct BreadcrumbBarExample {
    path: PathBuf,
}

impl Default for BreadcrumbBarExample {
    fn default() -> Self {
        Self {
            path: "/home/tibor/git/egui_extras_xt/Cargo.toml".into(),
        }
    }
}

impl eframe::App for BreadcrumbBarExample {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            breadcrumb_bar(ui, &mut self.path);
            ui.separator();

            if ui.button("\u{1F504} Reset").clicked() {
                *self = Self::default();
            }
        });
    }
}

fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(vec2(640.0, 480.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Breadcrumb bar example",
        options,
        Box::new(|_| Box::<BreadcrumbBarExample>::default()),
    );
}