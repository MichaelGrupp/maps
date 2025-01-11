use eframe::egui;
use egui_file_dialog::DialogState;

use crate::app::{ActiveMovable, AppState, ViewMode};
use crate::movable::{DragDirection, Draggable, Rotatable};

const GRID_ZOOM_AMOUNT: f32 = 1.;

impl AppState {
    fn flip_lens(&mut self, skip: i32) {
        if self.options.view_mode != ViewMode::Aligned {
            // Toggle in non-overlapping views.
            self.options.lens.enabled = !self.options.lens.enabled;
            return;
        }
        // In overlapping aligned view, switch to the next visible maps, then disable.
        if !self.options.lens.enabled {
            self.options.lens.enabled = true;
            return;
        }
        let keys: Vec<&String> = self
            .maps
            .keys()
            .filter(|k| self.maps.get(*k).unwrap().visible)
            .collect();
        if keys.is_empty() {
            self.options.lens.enabled = false;
            return;
        }
        let active_lens = self.options.active_lens.get_or_insert(keys[0].clone());
        if let Some(active_index) = keys.iter().position(|k| *k == active_lens) {
            let next_index = if active_index == 0 && skip.is_negative() {
                // Rotate index with right click.
                keys.len() - 1
            } else {
                (active_index as i32 + skip) as usize % keys.len()
            };
            if next_index == 0 && skip.is_positive() {
                self.options.active_lens = None;
                self.options.lens.enabled = false;
            } else {
                *active_lens = keys[next_index].clone();
            }
        }
    }

    fn dialogs_open(&self) -> bool {
        self.load_meta_file_dialog.state() == DialogState::Open
            || self.load_map_pose_file_dialog.state() == DialogState::Open
            || self.save_map_pose_file_dialog.state() == DialogState::Open
    }

    fn text_editing(&self) -> bool {
        self.options.pose_edit.edit_map_frame || self.options.pose_edit.edit_root_frame
    }

    pub fn handle_key_shortcuts(&mut self, ui: &egui::Ui) {
        if self.dialogs_open() || self.text_editing() {
            return;
        }

        ui.input(|i| {
            if i.key_released(egui::Key::Escape) {
                self.options.menu_visible = false;
                self.options.settings_visible = false;
                self.options.lens.enabled = false;
            } else if i.key_released(egui::Key::L) || i.pointer.secondary_released() {
                self.flip_lens(1);
            }
            if i.key_released(egui::Key::K) || i.pointer.primary_released() {
                if self.options.lens.enabled {
                    self.flip_lens(-1);
                }
            }
            if i.key_released(egui::Key::M) {
                self.options.menu_visible = !self.options.menu_visible;
            }
            if i.key_released(egui::Key::O) {
                self.options.settings_visible = !self.options.settings_visible;
            }
            if i.key_released(egui::Key::G) {
                self.options.grid.lines_visible = !self.options.grid.lines_visible;
            }

            // Get the obects that can be currently dragged.
            let draggable: Option<&mut dyn Draggable> = match self.options.active_movable {
                ActiveMovable::MapPose => {
                    match self
                        .maps
                        .get_mut(self.options.pose_edit.selected_map.as_str())
                    {
                        Some(map) => Some(&mut map.pose),
                        None => None,
                    }
                }
                ActiveMovable::Grid => Some(&mut self.options.grid),
                _ => None,
            };

            let drag_amount = self.options.pose_edit.movable_amounts.drag;
            if let Some(draggable) = draggable {
                if i.key_down(egui::Key::W) {
                    draggable.drag_directed(drag_amount, DragDirection::Up);
                }
                if i.key_down(egui::Key::A) {
                    draggable.drag_directed(drag_amount, DragDirection::Left);
                }
                if i.key_down(egui::Key::S) {
                    draggable.drag_directed(drag_amount, DragDirection::Down);
                }
                if i.key_down(egui::Key::D) {
                    draggable.drag_directed(drag_amount, DragDirection::Right);
                }
            }

            let rotatable: Option<&mut dyn Rotatable> = match self.options.active_movable {
                ActiveMovable::MapPose => {
                    match self
                        .maps
                        .get_mut(self.options.pose_edit.selected_map.as_str())
                    {
                        Some(map) => Some(&mut map.pose),
                        None => None,
                    }
                }
                _ => None,
            };

            let rotation_amount = self.options.pose_edit.movable_amounts.rotate;
            if let Some(rotatable) = rotatable {
                if i.key_down(egui::Key::Q) {
                    rotatable.rotate_directed(rotation_amount, DragDirection::Left);
                }
                if i.key_down(egui::Key::E) {
                    rotatable.rotate_directed(rotation_amount, DragDirection::Right);
                }
            }

            if i.key_down(egui::Key::Minus) {
                self.options.grid.zoom(-GRID_ZOOM_AMOUNT);
            }
            if i.key_down(egui::Key::Plus) {
                self.options.grid.zoom(GRID_ZOOM_AMOUNT);
            }
        });
    }
}
