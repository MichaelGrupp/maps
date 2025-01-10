use eframe::egui;

use crate::app::{AppState, ViewMode};
use crate::grid_options::DragDirection;

const GRID_DRAG_AMOUNT: f32 = 10.;
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

    pub fn handle_key_shortcuts(&mut self, ui: &egui::Ui) {
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
            if i.key_down(egui::Key::W) {
                self.options
                    .grid
                    .drag_directed(GRID_DRAG_AMOUNT, DragDirection::Up);
            }
            if i.key_down(egui::Key::A) {
                self.options
                    .grid
                    .drag_directed(GRID_DRAG_AMOUNT, DragDirection::Left);
            }
            if i.key_down(egui::Key::S) {
                self.options
                    .grid
                    .drag_directed(GRID_DRAG_AMOUNT, DragDirection::Down);
            }
            if i.key_down(egui::Key::D) {
                self.options
                    .grid
                    .drag_directed(GRID_DRAG_AMOUNT, DragDirection::Right);
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
