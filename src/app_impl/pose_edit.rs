use std::f32::consts::PI;

use eframe::egui;

use crate::app::AppState;
use crate::app_impl::constants::SPACE;

impl AppState {
    pub fn pose_edit(&mut self, ui: &mut egui::Ui) {
        egui::ComboBox::from_label("")
            .selected_text(self.options.selected_map.clone())
            .show_ui(ui, |ui| {
                for (name, _) in &self.maps {
                    ui.selectable_value(&mut self.options.selected_map, name.clone(), name);
                }
            });

        let map_name = self.options.selected_map.clone();

        if !map_name.is_empty() {
            ui.add_space(SPACE);
            egui::Grid::new("pose_io_grid")
                .num_columns(2)
                .striped(false)
                .show(ui, |ui| {
                    self.save_map_pose_button(ui, map_name.as_str());
                    self.load_map_pose_button(ui, map_name.as_str());
                });
        }

        match self.maps.get_mut(&map_name) {
            Some(map) => {
                ui.vertical(|ui| {
                    ui.add_space(SPACE);
                    ui.label("x");
                    ui.add(egui::Slider::new(
                        &mut map.pose.translation.x,
                        -1000.0..=1000.0,
                    ));
                    ui.label("y");
                    ui.add(egui::Slider::new(
                        &mut map.pose.translation.y,
                        -1000.0..=1000.0,
                    ));
                    ui.label("θ");
                    ui.add(egui::Slider::new(&mut map.pose.rotation.yaw, -PI..=PI));
                });
            }
            None => {
                ui.label("Select a map to edit its pose.");
            }
        }

        if !map_name.is_empty() {
            ui.add_space(SPACE);
            egui::Grid::new("pose_buttons_grid")
                .num_columns(2)
                .striped(false)
                .show(ui, |ui| {
                    if let Some(map) = self.maps.get_mut(&map_name) {
                        if ui.button("Reset").clicked() {
                            map.pose = Default::default();
                        }
                        if ui.button("Invert").clicked() {
                            map.pose.invert();
                        }
                    }
                });
        }
    }
}
