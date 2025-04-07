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
    pub(crate) fn pose_edit(&mut self, ui: &mut egui::Ui) {
        // ComboBox is in a horizontal scroll to not take too much space for long paths.
        // Waiting for: https://github.com/emilk/egui/discussions/1829
        egui::ScrollArea::horizontal().show(ui, |ui| {
            egui::ComboBox::from_label("")
                .selected_text(self.options.pose_edit.selected_map.clone())
                .show_ui(ui, |ui| {
                    for name in self.data.maps.keys() {
                        ui.selectable_value(
                            &mut self.options.pose_edit.selected_map,
                            name.clone(),
                            name,
                        );
                    }
                });
        });

        let map_name = self.options.pose_edit.selected_map.clone();
        let Some(map_pose) = self.data.maps.get_mut(&map_name).map(|m| &mut m.pose) else {
            ui.label("Select a map to edit its pose.");
            self.options.active_movable = ActiveMovable::Grid;
            return;
        };
        let original_map_pose = map_pose.clone();

        ui.add_space(SPACE);
        ui.separator();
        egui::Grid::new("pose_buttons_grid")
            .num_columns(2)
            .striped(false)
            .show(ui, |ui| {
                ui.selectable_value(
                    &mut self.options.active_movable,
                    ActiveMovable::MapPose,
                    "Move Map",
                )
                .on_hover_text(
                    "Toggle to move the selected map with the WASD keys, rotate with Q/E.",
                );
                ui.selectable_value(
                    &mut self.options.active_movable,
                    ActiveMovable::Grid,
                    "Move Grid",
                )
                .on_hover_text("Toggle to move the grid with the WASD keys.");
            });

        ui.add_space(SPACE);
        ui.vertical(|ui| {
            let movable_amounts = match self.options.active_movable {
                ActiveMovable::MapPose => &mut self.options.pose_edit.movable_amounts,
                ActiveMovable::Grid => &mut self.options.grid.movable_amounts,
                _ => unreachable!(),
            };
            ui.label("x/y step (m)")
                .on_hover_text("The amount of translation per key press.");
            ui.add(egui::Slider::new(&mut movable_amounts.drag, 0.0..=10.0));
            if self.options.active_movable == ActiveMovable::MapPose {
                ui.end_row();
                ui.label("θ step (rad)")
                    .on_hover_text("The amount of rotation per key press.");
                ui.add(egui::Slider::new(&mut movable_amounts.rotate, 0.0..=0.1));
            }
            ui.add_space(SPACE);
            ui.horizontal(|ui| {
                if ui
                    .button("Fine")
                    .on_hover_text("Quick setting for slow, fine movement.")
                    .clicked()
                {
                    *movable_amounts = MovableAmounts::PRESET_FINE;
                }
                if ui
                    .button("Medium")
                    .on_hover_text("Quick setting for medium step movements.")
                    .clicked()
                {
                    *movable_amounts = MovableAmounts::PRESET_MEDIUM;
                }
                if ui
                    .button("Coarse")
                    .on_hover_text("Quick setting for fast, coarse movement.")
                    .clicked()
                {
                    *movable_amounts = MovableAmounts::PRESET_COARSE;
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
                    ui.label("Root frame ID:").on_hover_text(
                        "The name of the coordinate frame that the map pose is relative to. \
                        Can be left empty if it's not needed for your use case.",
                    );
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

                    ui.label("Map frame ID:").on_hover_text(
                        "The name of the map's origin coordinate frame. \
                        Can be left empty if it's not needed for your use case.",
                    );
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

        if original_map_pose != *map_pose {
            self.status.unsaved_changes = true;
        }

        egui::Grid::new("pose_io_grid")
            .num_columns(2)
            .striped(false)
            .show(ui, |ui| {
                self.load_map_pose_button(ui, map_name.as_str());
                self.save_map_pose_button(ui, map_name.as_str());
            });
    }

    pub(crate) fn apply_pose_to_other_maps(&mut self, ui: &mut egui::Ui) {
        ui.label("Apply pose also to:");
        ui.add_space(SPACE);
        let mut selected_maps: Vec<String> = Vec::new();
        egui::Grid::new("pose_apply_grid")
            .num_columns(2)
            .striped(false)
            .show(ui, |ui| {
                for name in self.data.maps.keys() {
                    if name == &self.options.pose_edit.selected_map {
                        continue;
                    }
                    if ui
                        .button(name)
                        .on_hover_text("Click to copy the map pose also to this other map.")
                        .clicked()
                    {
                        selected_maps.push(name.clone());
                    }
                    ui.end_row();
                }
            });

        if selected_maps.is_empty() {
            return;
        }

        self.status.unsaved_changes = true;
        let map_pose_to_copy = self.data.maps[&self.options.pose_edit.selected_map]
            .pose
            .clone();

        for map_name in selected_maps {
            let Some(map_pose) = self.data.maps.get_mut(&map_name).map(|m| &mut m.pose) else {
                continue;
            };
            *map_pose = map_pose_to_copy.clone();
        }
    }
}
