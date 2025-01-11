use std::f32::consts::PI;

use eframe::egui;

use crate::app::{ActiveMovable, AppState};
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
        let map_pose = self.maps.get_mut(&map_name).map(|m| &mut m.pose);
        if map_pose.is_none() {
            ui.label("Select a map to edit its pose.");
            return;
        }
        let map_pose = map_pose.unwrap();

        ui.add_space(SPACE);
        egui::Grid::new("pose_buttons_grid")
            .num_columns(2)
            .striped(false)
            .show(ui, |ui| {
                if ui.button("Zero").clicked() {
                    *map_pose = Default::default();
                }
                if ui.button("Invert").clicked() {
                    map_pose.invert();
                }
                ui.end_row();
                ui.end_row();
                ui.selectable_value(
                    &mut self.options.active_movable,
                    ActiveMovable::MapPose,
                    "Move Map",
                );
                ui.selectable_value(
                    &mut self.options.active_movable,
                    ActiveMovable::Grid,
                    "Move Grid",
                );
            });

        ui.vertical(|ui| {
            ui.add_space(SPACE);
            ui.label("x");
            ui.add(egui::Slider::new(
                &mut map_pose.translation.x,
                -1000.0..=1000.0,
            ));
            ui.label("y");
            ui.add(egui::Slider::new(
                &mut map_pose.translation.y,
                -1000.0..=1000.0,
            ));
            ui.label("θ");
            ui.add(egui::Slider::new(&mut map_pose.rotation.yaw, -PI..=PI));
        });

        ui.add_space(2. * SPACE);
        egui::Grid::new("pose_io_grid")
            .num_columns(2)
            .striped(false)
            .show(ui, |ui| {
                self.save_map_pose_button(ui, map_name.as_str());
                self.load_map_pose_button(ui, map_name.as_str());
            });
    }
}
