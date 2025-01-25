use eframe::egui;

use crate::app::AppState;
use crate::app_impl::constants::SPACE;

impl AppState {
    pub fn menu_panel(&mut self, ui: &mut egui::Ui) {
        if !self.options.menu_visible {
            return;
        }
        egui::SidePanel::left("menu").show(ui.ctx(), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                self.menu_content(ui);
            });
        });
    }
    fn menu_content(&mut self, ui: &mut egui::Ui) {
        ui.heading("Maps");
        ui.add_space(SPACE);
        self.load_meta_button(ui);
        ui.separator();
        let mut to_delete: Vec<String> = Vec::new();
        egui::Grid::new("maps_list")
            .num_columns(3)
            .striped(true)
            .show(ui, |ui| {
                for (name, map) in &mut self.maps {
                    if ui.checkbox(&mut map.visible, name).changed() {
                        self.tile_manager.set_visible(name, map.visible);
                    }
                    if ui.button("🗑").on_hover_text("Delete Map").clicked() {
                        to_delete.push(name.clone());
                    }
                    if map.meta.origin_theta.angle() != 0. {
                        ui.label(
                            egui::RichText::new("⚠")
                                .strong()
                                .color(egui::Color32::ORANGE),
                        )
                        .on_hover_text(
                            "This map has a non-zero origin rotation component in its metadata.\n\
                            maps uses it, but it's not supported by most ROS tools.\n\n\
                            It's recommended to save alignment transformations separately,\n\
                            e.g. using the Pose editor here.",
                        );
                    }
                    ui.end_row();
                }
            });
        self.delete(&to_delete);

        if self.maps.is_empty() {
            return;
        }

        ui.separator();
        ui.add_space(SPACE);
        ui.heading("Pose");
        ui.add_space(SPACE);
        self.pose_edit(ui);
        if !self.options.pose_edit.selected_map.is_empty() && self.maps.len() > 1 {
            ui.separator();
            ui.add_space(SPACE);
            self.apply_pose_to_other_maps(ui);
        }
    }
}
