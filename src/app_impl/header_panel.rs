use eframe::egui;

use crate::app::{AppState, ViewMode};
use crate::app_impl::constants::ICON_SIZE;

impl AppState {
    fn view_buttons(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_centered(|ui| {
            ui.selectable_value(&mut self.options.view_mode, ViewMode::Tiles, "Tiles")
                .on_hover_text("Show the maps in separate tab tiles that can be rearranged.");
            ui.selectable_value(&mut self.options.view_mode, ViewMode::Stacked, "Stacked")
                .on_hover_text("Show the maps stacked on top of each other.");
            ui.selectable_value(&mut self.options.view_mode, ViewMode::Aligned, "Aligned")
                .on_hover_text("Show the maps in a shared coordinate system.");
        });
    }

    pub fn header_panel(&mut self, ui: &mut egui::Ui) {
        let add_toggle_button = |ui: &mut egui::Ui,
                                 icon: &str,
                                 tooltip: &str,
                                 switch: &mut bool| {
            if ui
                .add_sized(
                    egui::vec2(ICON_SIZE, ICON_SIZE),
                    egui::SelectableLabel::new(*switch, egui::RichText::new(icon).size(ICON_SIZE)),
                )
                .on_hover_text(tooltip)
                .clicked()
            {
                *switch = !*switch;
            }
        };

        egui::TopBottomPanel::new(egui::containers::panel::TopBottomSide::Top, "header").show(
            ui.ctx(),
            |ui| {
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                        add_toggle_button(ui, "☰", "Show Menu", &mut self.options.menu_visible);
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        add_toggle_button(
                            ui,
                            "⚙",
                            "Show Settings",
                            &mut self.options.settings_visible,
                        );
                        ui.add_space(ICON_SIZE);
                        self.view_buttons(ui);
                    });
                });
            },
        );
    }
}
