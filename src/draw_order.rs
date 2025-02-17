use eframe::egui;
use egui_dnd::dnd;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct DrawOrder {
    keys: Vec<String>,
}

impl DrawOrder {
    pub fn add(&mut self, name: String) {
        self.keys.push(name);
    }

    pub fn remove(&mut self, name: &str) {
        self.keys.retain(|x| x != name);
    }

    pub fn keys(&self) -> &Vec<String> {
        self.keys.as_ref()
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        dnd(ui, "draw_order").show_vec(&mut self.keys, |ui, item, handle, state| {
            ui.horizontal(|ui| {
                handle.ui(ui, |ui| {
                    ui.label(egui::RichText::new(state.index.to_string()).strong());
                    ui.label(item.clone());
                });
            });
        });
    }
}
