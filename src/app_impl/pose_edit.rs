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
            self.options.active_movable = ActiveMovable::Grid;
            return;
        }
        let map_pose = map_pose.unwrap();

        ui.add_space(SPACE);
        egui::Grid::new("pose_buttons_grid")
            .num_columns(2)
            .striped(false)
            .show(ui, |ui| {
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

        ui.separator();
        ui.vertical(|ui| {
            ui.label("x/y step (m)");
            ui.add(egui::Slider::new(
                &mut self.options.movable_amounts.drag,
                0.0..=10.0,
            ));
            if self.options.active_movable == ActiveMovable::MapPose {
                ui.end_row();
                ui.label("θ step (rad)");
                ui.add(egui::Slider::new(
                    &mut self.options.movable_amounts.rotate,
                    0.0..=0.1,
                ));
            }
        });

        ui.separator();
        ui.vertical(|ui| {
            ui.add_space(SPACE);
            ui.horizontal(|ui| {
                if ui.button("Zero values").clicked() {
                    *map_pose = Default::default();
                }
                if ui.button("Invert pose").clicked() {
                    map_pose.invert();
                }
            });
            ui.add_space(SPACE);
            ui.label("x (m)");
            ui.add(egui::Slider::new(
                &mut map_pose.translation.x,
                -1000.0..=1000.0,
            ));
            ui.label("y (m)");
            ui.add(egui::Slider::new(
                &mut map_pose.translation.y,
                -1000.0..=1000.0,
            ));
            ui.label("θ (rad)");
            ui.add(egui::Slider::new(&mut map_pose.rotation.yaw, -PI..=PI));
            ui.add_space(SPACE);
        });

        egui::Grid::new("pose_io_grid")
            .num_columns(2)
            .striped(false)
            .show(ui, |ui| {
                self.save_map_pose_button(ui, map_name.as_str());
                self.load_map_pose_button(ui, map_name.as_str());
            });
    }
}
