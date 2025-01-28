use eframe::egui;

use crate::app::AppState;
use crate::app_impl::constants::SPACE;

impl AppState {
    pub fn quit_modal(&mut self, ui: &mut egui::Ui) {
        if !self.status.quit_modal_active || self.data.maps.is_empty() {
            return;
        }

        egui::Modal::new(egui::Id::new("Confirm Exit")).show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new("⚠").size(50.));
                ui.label("There seem to be unsaved changes. Are you sure you want to quit?");
                ui.label("You can save the session in the menu.");
            });
            ui.add_space(SPACE);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Cancel").clicked() {
                    self.status.quit_modal_active = false;
                } else if ui.button("Quit").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    self.status.quit_modal_active = false;
                    self.status.unsaved_changes = false;
                }
            });
        });
    }
}
