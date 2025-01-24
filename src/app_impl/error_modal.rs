use eframe::egui;

use crate::app::AppState;
use crate::app_impl::constants::SPACE;

impl AppState {
    pub fn error_modal(&mut self, ui: &mut egui::Ui) {
        if self.status.error.is_empty() {
            return;
        }

        let bloodbath = egui::Color32::from_rgba_unmultiplied(255, 0, 0, 50);
        egui::Modal::new(egui::Id::new("Error"))
            .backdrop_color(bloodbath)
            .show(ui.ctx(), |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new("💩").size(50.));
                });
                ui.code(self.status.error.clone());
                ui.add_space(SPACE);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Close").clicked() {
                        self.status.error.clear();
                    }
                });
            });
    }
}
