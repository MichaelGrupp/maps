use eframe::egui;

use crate::app::AppState;
use crate::app_impl::constants::SPACE;

impl AppState {
    fn keybinding_table(ui: &mut egui::Ui) {
        // Collapsible table of keybindings.
        egui::CollapsingHeader::new("Keybindings")
            .default_open(false)
            .show(ui, |ui| {
                egui::Grid::new("keybindings")
                    .striped(true)
                    .max_col_width(ui.available_width())
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("Key").strong());
                        ui.label(egui::RichText::new("Action").strong());
                        ui.end_row();

                        // Movements
                        ui.label("+/-");
                        ui.label("Zoom grid in 'Aligned' view.");
                        ui.end_row();
                        ui.label("w/a/s/d");
                        ui.label(
                            "Move grid in 'Aligned' view, or move selected map \
                    when editing a pose and 'Move Map' is selected.",
                        );
                        ui.end_row();
                        ui.label("q/e");
                        ui.label(
                            "Rotate selected map in 'Aligned' view when editing \
                    a pose and 'Move Map' is selected.",
                        );
                        ui.end_row();
                        ui.end_row();

                        // Mouse
                        ui.label("click + drag");
                        ui.label("Move grid in 'Aligned' view. Move tabs/panes in 'Tiles' view.");
                        ui.end_row();
                        ui.label("scroll");
                        ui.label(
                            "Zoom grid in 'Aligned' view or change lens size in \
                        'Tiles'/'Stacked' view, if active.",
                        );
                        ui.end_row();
                        ui.label("left click / l");
                        ui.label("Toggle hovering lens.");
                        ui.end_row();
                        ui.label("g");
                        ui.label("Toggle grid lines in 'Aligned' view.");
                        ui.end_row();
                        ui.end_row();

                        // General
                        ui.label("m");
                        ui.label("Toggle the menu sidebar.");
                        ui.end_row();
                        ui.label("o");
                        ui.label("Toggle the settings sidebar.");
                        ui.end_row();
                        ui.label("Esc");
                        ui.label(
                            "Close open sidebars or child windows. Deactivate lens if active.",
                        );
                        ui.end_row();
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
                        ui.label(self.build_info.clone());
                        ui.separator();
                        Self::keybinding_table(ui);
                    });
                });
        }
    }
}
