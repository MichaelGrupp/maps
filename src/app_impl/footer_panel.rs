use eframe::egui;

use crate::app::AppState;

impl AppState {
    pub fn footer_panel(&mut self, ui: &mut egui::Ui) {
        egui::TopBottomPanel::new(egui::containers::panel::TopBottomSide::Bottom, "footer").show(
            ui.ctx(),
            |ui| {
                ui.horizontal(|ui| ui.label(self.status_message.clone()));
            },
        );
    }
}
