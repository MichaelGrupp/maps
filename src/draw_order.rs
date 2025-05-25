use eframe::egui;
use egui_dnd::dnd;
use serde::{Deserialize, Serialize};

use crate::app_impl::ui_helpers::display_path;

#[derive(Default, Serialize, Deserialize)]
pub struct DrawOrder {
    keys: Vec<String>,
}

impl DrawOrder {
    pub fn add(&mut self, name: String) {
        if self.contains(name.as_str()) {
            // Handle corner case of reloading a map that was already inserted.
            return;
        }
        self.keys.push(name);
    }

    pub fn remove(&mut self, name: &str) {
        self.keys.retain(|x| x != name);
    }

    pub fn keys(&self) -> &Vec<String> {
        self.keys.as_ref()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.keys.contains(&name.to_string())
    }

    pub fn extend(&mut self, other: &DrawOrder) {
        for name in other.keys.iter() {
            self.add(name.clone());
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, show_full_paths: bool) {
        dnd(ui, "draw_order").show_vec(&mut self.keys, |ui, item, handle, state| {
            ui.horizontal(|ui| {
                handle.ui(ui, |ui| {
                    ui.label(egui::RichText::new(state.index.to_string()).strong());
                    ui.label(display_path(item, show_full_paths));
                });
            });
        });
    }
}
