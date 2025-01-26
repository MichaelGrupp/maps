use eframe::egui;
use serde::{Deserialize, Serialize};

use crate::app::AppState;
use crate::app_impl::constants::SPACE;

#[derive(Debug, Serialize, Deserialize)]
pub struct CanvasOptions {
    pub background_color: egui::Color32,
}

impl Default for CanvasOptions {
    fn default() -> Self {
        Self {
            background_color: egui::Visuals::default().faint_bg_color,
        }
    }
}

impl AppState {
    pub fn canvas_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("Canvas");
        if ui.button("Reset").clicked() {
            self.options.canvas_settings = CanvasOptions::default();
        }
        ui.add_space(SPACE);
        ui.end_row();
        ui.label("Dark / Light mode");
        egui::widgets::global_theme_preference_switch(ui);
        ui.end_row();
        ui.label("Background color");
        ui.color_edit_button_srgba(&mut self.options.canvas_settings.background_color);
    }
}
