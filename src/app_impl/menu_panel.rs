use eframe::egui;

use crate::app::AppState;
use crate::app_impl::constants::SPACE;

impl AppState {
    pub fn menu_panel(&mut self, ui: &mut egui::Ui) {
        if !self.options.menu_visible {
            return;
        }
        egui::SidePanel::left("menu").show(ui.ctx(), |ui| {
            ui.heading("Maps");
            ui.add_space(SPACE);
            self.load_meta_button(ui);
            ui.separator();
            let mut to_delete: Vec<String> = Vec::new();
            egui::Grid::new("maps_list")
                .num_columns(2)
                .striped(true)
                .show(ui, |ui| {
                    for (name, map) in &mut self.maps {
                        if ui.checkbox(&mut map.visible, name).changed() {
                            self.tile_manager.set_visible(name, map.visible);
                        }
                        if ui.button("🗑").on_hover_text("Delete Map").clicked() {
                            to_delete.push(name.clone());
                        }
                        ui.end_row();
                    }
                });
            for name in to_delete {
                self.maps.remove(&name);
                self.tile_manager.remove_pane(&name);
                if let Some(active_lens) = &self.options.active_lens {
                    if active_lens == &name {
                        self.options.active_lens = None;
                    }
                }
                if let Some(active_tint_selection) =
                    &self.options.tint_settings.active_tint_selection
                {
                    if active_tint_selection == &name {
                        self.options.tint_settings.active_tint_selection = None;
                    }
                }
            }
        });
    }
}
