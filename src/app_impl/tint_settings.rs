use eframe::egui;

use crate::app::AppState;
use crate::app_impl::constants::SPACE;

use crate::texture_request::NO_TINT;

impl AppState {
    pub fn tint_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("Blend");
        ui.add_space(SPACE);
        ui.end_row();

        let all_key = "< All >".to_string();
        let selected = self
            .options
            .active_tint_selection
            .get_or_insert(all_key.clone());
        egui::ComboBox::from_label("")
            .selected_text(format!("{}", selected))
            .show_ui(ui, |ui| {
                ui.selectable_value(selected, all_key.clone(), &all_key);
                for name in self.maps.keys() {
                    ui.selectable_value(selected, name.to_string(), name);
                }
            });

        let reset = ui.button("Reset").clicked();
        ui.end_row();

        ui.label("Tint color / alpha");
        if *selected == all_key {
            let tint = &mut self.options.tint_for_all;
            if reset {
                *tint = NO_TINT;
            }
            ui.color_edit_button_srgba(tint);
            for map in self.maps.values_mut() {
                map.tint = Some(*tint);
            }
        } else {
            let tint = self
                .maps
                .get_mut(selected)
                .unwrap()
                .tint
                .get_or_insert(NO_TINT);
            if reset {
                *tint = NO_TINT;
            }
            ui.color_edit_button_srgba(tint);
        }
    }
}
