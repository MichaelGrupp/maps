use eframe::egui;

use crate::app::{AppState, ViewMode};

impl AppState {
    pub fn settings_panel(&mut self, ui: &mut egui::Ui) {
        if !self.options.settings_visible {
            return;
        }
        egui::SidePanel::right("settings").show(ui.ctx(), |ui| {
            egui::Grid::new("settings_grid")
                .num_columns(2)
                .striped(false)
                .show(ui, |ui| {
                    self.canvas_settings(ui);
                    ui.end_row();
                    ui.end_row();

                    self.lens_settings(ui);

                    if self.options.view_mode == ViewMode::Aligned {
                        ui.end_row();
                        ui.end_row();
                        self.grid_settings(ui);
                    }

                    if !self.maps.is_empty() {
                        ui.end_row();
                        ui.end_row();
                        self.tint_settings(ui);
                    }
                });
        });
    }
}
