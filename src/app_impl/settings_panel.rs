use eframe::egui;

use crate::app::{AppState, ViewMode};

impl AppState {
    pub(crate) fn settings_panel(&mut self, ui: &mut egui::Ui) {
        if !self.options.settings_visible {
            return;
        }
        egui::SidePanel::right("settings").show(ui.ctx(), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("settings_grid")
                    .num_columns(2)
                    .striped(false)
                    .show(ui, |ui| {
                        ui.heading("App");
                        ui.end_row();
                        ui.label("Autosave options").on_hover_text(
                            "Save the app options when the window is closed.\n\
                        The options are loaded when the app is started.",
                        );
                        ui.checkbox(&mut self.options.persistence.autosave, "");
                        ui.end_row();
                        ui.end_row();

                        self.canvas_settings(ui);
                        ui.end_row();
                        ui.end_row();

                        if !self.data.maps.is_empty() {
                            self.tint_settings(ui);
                            ui.end_row();
                            ui.end_row();
                        }

                        if self.options.view_mode != ViewMode::Aligned {
                            self.lens_settings(ui);
                            ui.end_row();
                            ui.end_row();
                        }

                        if self.options.view_mode == ViewMode::Aligned {
                            self.grid_settings(ui);
                            ui.end_row();
                            ui.end_row();
                        }
                    });
            });
        });
    }
}
