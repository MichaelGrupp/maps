use eframe::egui;

use crate::app::AppState;
use crate::app_impl::constants::SPACE;

#[derive(Debug)]
pub struct CanvasSettings {
    pub background_color: egui::Color32,
}

impl Default for CanvasSettings {
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
            self.options.canvas_settings = CanvasSettings::default();
        }
        ui.add_space(SPACE);
        ui.end_row();
        ui.label("Background color");
        ui.color_edit_button_srgba(&mut self.options.canvas_settings.background_color);
    }
}
