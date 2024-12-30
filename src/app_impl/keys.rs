use eframe::egui;

use crate::app::{AppState, ViewMode};

impl AppState {
    fn flip_lens(&mut self) {
        if !self.options.lens.enabled || self.options.view_mode != ViewMode::Aligned {
            // Activate in all cases, or toggle in non-overlapping views.
            self.options.lens.enabled = !self.options.lens.enabled;
            return;
        }
        // In overlapping aligned view, switch to the next visible maps, then disable.
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
            let next_index = (active_index + 1) % keys.len();
            if next_index == 0 {
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
                self.flip_lens();
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
                self.options.grid.offset.y -= 10.;
            }
            if i.key_down(egui::Key::A) {
                self.options.grid.offset.x -= 10.;
            }
            if i.key_down(egui::Key::S) {
                self.options.grid.offset.y += 10.;
            }
            if i.key_down(egui::Key::D) {
                self.options.grid.offset.x += 10.;
            }
            if i.key_down(egui::Key::Minus) {
                self.options.grid.scale = (self.options.grid.scale - 1.)
                    .clamp(self.options.grid.min_scale, self.options.grid.max_scale);
            }
            if i.key_down(egui::Key::Plus) {
                self.options.grid.scale = (self.options.grid.scale + 1.)
                    .clamp(self.options.grid.min_scale, self.options.grid.max_scale);
            }
        });
    }
}
