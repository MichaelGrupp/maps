use std::f32::consts::PI;

use eframe::egui;

use crate::app::{ActiveMovable, AppState};
use crate::app_impl::constants::SPACE;
use crate::app_impl::ui_helpers::{button_row, display_path, fixed_label};
use crate::movable::MovableAmounts;

/// Width reserved for the value field next to a full-width slider / text field.
const VALUE_FIELD_WIDTH: f32 = 64.;

#[derive(Debug, Default)]
pub struct PoseEditOptions {
    pub selected_map: String,
    pub edit_root_frame: bool,
    pub edit_map_frame: bool,
    pub movable_amounts: MovableAmounts,
}

impl AppState {
    pub(crate) fn pose_edit_combo_box(&mut self, ui: &mut egui::Ui) {
        if self.data.maps.is_empty() {
            self.options.active_movable = ActiveMovable::Grid;
            return;
        }
        // Default to first map if the movable is changed and there's nothing selected yet.
        if self.options.active_movable == ActiveMovable::MapPose
            && self.options.pose_edit.selected_map.is_empty()
        {
            self.options.pose_edit.selected_map =
                self.data.maps.keys().next().cloned().unwrap_or_default();
        }
        // Show combo box to select the map for pose editing.
        egui::ComboBox::from_label("")
            .selected_text(display_path(
                &self.options.pose_edit.selected_map,
                self.options.display.show_full_paths,
            ))
            .show_ui(ui, |ui| {
                for name in self.data.maps.keys() {
                    ui.selectable_value(
                        &mut self.options.pose_edit.selected_map,
                        name.clone(),
                        display_path(name, self.options.display.show_full_paths),
                    )
                    .on_hover_text(name);
                }
            });
    }

    pub(crate) fn pose_edit(&mut self, ui: &mut egui::Ui) {
        self.pose_edit_combo_box(ui);

        let map_name = self.options.pose_edit.selected_map.clone();
        let Some(original_map_pose) = self.data.maps.get(&map_name).map(|m| m.pose.clone()) else {
            ui.label("Select a map to edit its pose.");
            self.options.active_movable = ActiveMovable::Grid;
            return;
        };

        // Match the sliders' label column to the button column width for nicer alignment.
        // (a third of the width)
        let spacing_x = ui.spacing().item_spacing.x;
        let label_width = ((ui.available_width() - 2. * spacing_x) / 3.).max(0.);
        let slider_width =
            (ui.available_width() - label_width - spacing_x - VALUE_FIELD_WIDTH).max(0.);
        ui.spacing_mut().slider_width = slider_width;

        ui.add_space(SPACE);
        ui.separator();
        ui.columns(3, |columns| {
            columns[0].horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new("Key controls").strong().underline())
                    .on_hover_text("Select what can be moved via keyboard (WASD/QE).");
            });
            columns[1].vertical_centered_justified(|ui| {
                ui.selectable_value(
                    &mut self.options.active_movable,
                    ActiveMovable::MapPose,
                    "Move Map",
                )
                .on_hover_text(
                    "Toggle to move the selected map with the WASD keys, rotate with Q/E.",
                );
            });
            columns[2].vertical_centered_justified(|ui| {
                ui.selectable_value(
                    &mut self.options.active_movable,
                    ActiveMovable::Grid,
                    "Move Grid",
                )
                .on_hover_text("Toggle to move the grid with the WASD keys.");
            });
        });

        ui.add_space(SPACE);
        ui.vertical(|ui| {
            let movable_amounts = match self.options.active_movable {
                ActiveMovable::MapPose => &mut self.options.pose_edit.movable_amounts,
                ActiveMovable::Grid => &mut self.options.grid.movable_amounts,
                _ => unreachable!(),
            };
            match button_row(
                ui,
                &[
                    ("Fine", "Quick setting for slow, fine movement."),
                    ("Medium", "Quick setting for medium step movements."),
                    ("Coarse", "Quick setting for fast, coarse movement."),
                ],
            ) {
                Some(0) => *movable_amounts = MovableAmounts::PRESET_FINE,
                Some(1) => *movable_amounts = MovableAmounts::PRESET_MEDIUM,
                Some(2) => *movable_amounts = MovableAmounts::PRESET_COARSE,
                _ => {}
            }
            ui.add_space(SPACE);
            ui.horizontal(|ui| {
                fixed_label(ui, label_width, "x/y step (m)")
                    .on_hover_text("The amount of translation per key press.");
                ui.add(egui::Slider::new(&mut movable_amounts.drag, 0.0..=10.0));
            });
            if self.options.active_movable == ActiveMovable::MapPose {
                ui.horizontal(|ui| {
                    fixed_label(ui, label_width, "θ step (rad)")
                        .on_hover_text("The amount of rotation per key press.");
                    ui.add(egui::Slider::new(&mut movable_amounts.rotate, 0.0..=0.1));
                });
            }
        });

        ui.separator();
        ui.vertical(|ui| {
            ui.columns(3, |columns| {
                columns[0].horizontal(|ui| {
                    ui.label(egui::RichText::new("Values").strong().underline());
                });
                columns[1].vertical_centered_justified(|ui| {
                    self.load_map_pose_button(ui, map_name.as_str());
                });
                columns[2].vertical_centered_justified(|ui| {
                    self.save_map_pose_button(ui, map_name.as_str());
                });
            });

            ui.add_space(SPACE);

            let Some(map_pose) = self.data.maps.get_mut(&map_name).map(|m| &mut m.pose) else {
                return;
            };
            match button_row(
                ui,
                &[
                    ("Reset", "Resets all values."),
                    ("Negate", "Negates the pose components."),
                    ("Invert", "Inverts the pose."),
                ],
            ) {
                Some(0) => *map_pose = Default::default(),
                Some(1) => map_pose.negate(),
                Some(2) => map_pose.invert(),
                _ => {}
            }
            ui.add_space(SPACE);
            ui.horizontal(|ui| {
                fixed_label(ui, label_width, "x (m)");
                ui.add(egui::Slider::new(
                    &mut map_pose.translation.x,
                    -1000.0..=1000.0,
                ));
            });
            ui.horizontal(|ui| {
                fixed_label(ui, label_width, "y (m)");
                ui.add(egui::Slider::new(
                    &mut map_pose.translation.y,
                    -1000.0..=1000.0,
                ));
            });
            ui.horizontal(|ui| {
                fixed_label(ui, label_width, "θ (rad)");
                ui.add(egui::Slider::new(&mut map_pose.rotation.yaw, -PI..=PI));
            });
            ui.add_space(SPACE);

            ui.horizontal(|ui| {
                fixed_label(ui, label_width, "Root frame ID:").on_hover_text(
                    "The name of the coordinate frame that the map pose is relative to. \
                    Can be left empty if it's not needed for your use case.",
                );
                if self.options.pose_edit.edit_root_frame {
                    // Match text edit width with surrounding sliders.
                    ui.add(
                        egui::TextEdit::singleline(&mut map_pose.root_frame)
                            .desired_width(slider_width),
                    );
                } else {
                    fixed_label(ui, slider_width, &map_pose.root_frame);
                }
                ui.toggle_value(&mut self.options.pose_edit.edit_root_frame, "✏");
            });
            ui.horizontal(|ui| {
                fixed_label(ui, label_width, "Map frame ID:").on_hover_text(
                    "The name of the map's origin coordinate frame. \
                    Can be left empty if it's not needed for your use case.",
                );
                if self.options.pose_edit.edit_map_frame {
                    ui.add(
                        egui::TextEdit::singleline(&mut map_pose.map_frame)
                            .desired_width(slider_width),
                    );
                } else {
                    fixed_label(ui, slider_width, &map_pose.map_frame);
                }
                ui.toggle_value(&mut self.options.pose_edit.edit_map_frame, "✏");
            });
        });

        if self
            .data
            .maps
            .get(&map_name)
            .is_some_and(|m| m.pose != original_map_pose)
        {
            self.status.unsaved_changes = true;
        }
    }

    pub(crate) fn apply_pose_to_other_maps(&mut self, ui: &mut egui::Ui) {
        ui.label("Apply pose also to:");
        ui.add_space(SPACE);
        let mut selected_maps: Vec<String> = Vec::new();
        ui.vertical_centered_justified(|ui| {
            for name in self.data.maps.keys() {
                if name == &self.options.pose_edit.selected_map {
                    continue;
                }
                if ui
                    .button(display_path(name, self.options.display.show_full_paths))
                    .on_hover_text(format!("Click to copy the map pose also to {name}."))
                    .clicked()
                {
                    selected_maps.push(name.clone());
                }
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
