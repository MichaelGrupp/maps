use std::f32::consts::PI;

use eframe::egui;

use crate::app::{ActiveMovable, AppState};
use crate::app_impl::constants::SPACE;
use crate::movable::MovableAmounts;

#[derive(Debug, Default)]
pub struct PoseEditOptions {
    pub selected_map: String,
    pub edit_root_frame: bool,
    pub edit_map_frame: bool,
    pub movable_amounts: MovableAmounts,
}

impl AppState {
    pub fn pose_edit(&mut self, ui: &mut egui::Ui) {
        egui::ComboBox::from_label("")
            .selected_text(self.options.pose_edit.selected_map.clone())
            .show_ui(ui, |ui| {
                for (name, _) in &self.maps {
                    ui.selectable_value(
                        &mut self.options.pose_edit.selected_map,
                        name.clone(),
                        name,
                    );
                }
            });

        let map_name = self.options.pose_edit.selected_map.clone();
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
                &mut self.options.pose_edit.movable_amounts.drag,
                0.0..=10.0,
            ));
            if self.options.active_movable == ActiveMovable::MapPose {
                ui.end_row();
                ui.label("θ step (rad)");
                ui.add(egui::Slider::new(
                    &mut self.options.pose_edit.movable_amounts.rotate,
                    0.0..=0.1,
                ));
            }
            ui.add_space(SPACE);
            ui.horizontal(|ui| {
                if ui.button("Fine").clicked() {
                    self.options.pose_edit.movable_amounts.drag = 0.001;
                    self.options.pose_edit.movable_amounts.rotate = 0.001;
                }
                if ui.button("Medium").clicked() {
                    self.options.pose_edit.movable_amounts.drag = 0.1;
                    self.options.pose_edit.movable_amounts.rotate = 0.01;
                }
                if ui.button("Coarse").clicked() {
                    self.options.pose_edit.movable_amounts.drag = 1.;
                    self.options.pose_edit.movable_amounts.rotate = 0.1;
                }
            });
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

            egui::Grid::new("pose_frame_ids_grid")
                .max_col_width(ui.available_width())
                .num_columns(3)
                .striped(false)
                .show(ui, |ui| {
                    ui.label("Root frame ID:");
                    if self.options.pose_edit.edit_root_frame {
                        // Sized because otherwise it goes too wide (?).
                        ui.add_sized(
                            egui::vec2(80., 20.),
                            egui::widgets::TextEdit::singleline(&mut map_pose.root_frame),
                        );
                    } else {
                        ui.label(map_pose.root_frame.clone());
                    }
                    ui.toggle_value(&mut self.options.pose_edit.edit_root_frame, "✏");
                    ui.end_row();

                    ui.label("Map frame ID:");
                    if self.options.pose_edit.edit_map_frame {
                        ui.add_sized(
                            egui::vec2(80., 20.),
                            egui::widgets::TextEdit::singleline(&mut map_pose.map_frame),
                        );
                    } else {
                        ui.label(map_pose.map_frame.clone());
                    }
                    ui.toggle_value(&mut self.options.pose_edit.edit_map_frame, "✏");
                });
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
