use eframe::egui;

use crate::app::AppState;
use crate::app_impl::constants::SPACE;

impl AppState {
    fn keybinding_table(ui: &mut egui::Ui) {
        // Collapsible table of keybindings.
        egui::CollapsingHeader::new("Keybindings")
            .default_open(false)
            .show(ui, |ui| {
                egui::Grid::new("keybindings").striped(true).show(ui, |ui| {
                    ui.label("Key");
                    ui.label("Action");
                    ui.end_row();
                    ui.label("TODO");
                });
            });
    }

    pub fn info_window(&mut self, ui: &mut egui::Ui) {
        if self.options.help_visible {
            egui::Window::new("Info")
                .open(&mut self.options.help_visible)
                .pivot(egui::Align2::CENTER_CENTER)
                .default_pos(ui.ctx().used_rect().center())
                .fixed_size(egui::vec2(500., 500.))
                .show(ui.ctx(), |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("maps");
                        ui.add_space(SPACE);
                        ui.hyperlink_to(
                            egui::RichText::new(egui::special_emojis::GITHUB).heading(),
                            "https://www.github.com/MichaelGrupp/maps",
                        );
                        ui.add_space(SPACE);
                        ui.label("© Michael Grupp. Licensed under the Apache 2.0 license.");
                        ui.separator();
                        Self::keybinding_table(ui);
                    });
                });
        }
    }
}
