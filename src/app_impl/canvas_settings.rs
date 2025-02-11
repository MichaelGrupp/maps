use eframe::egui;
use serde::{Deserialize, Serialize};

use crate::app::AppState;
use crate::app_impl::constants::SPACE;

fn default_theme_pref() -> egui::ThemePreference {
    // TODO: add default() to egui::ThemePreference
    // See: https://github.com/emilk/egui/pull/5702
    egui::ThemePreference::System
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CanvasOptions {
    pub background_color: egui::Color32,
    #[serde(default = "default_theme_pref")]
    pub theme_preference: egui::ThemePreference,
}

impl Default for CanvasOptions {
    fn default() -> Self {
        Self {
            background_color: egui::Visuals::default().faint_bg_color,
            theme_preference: default_theme_pref(),
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
        // Theme is applied in main update(), to ensure it's also applied when this ui is hidden.
        self.options
            .canvas_settings
            .theme_preference
            .radio_buttons(ui);
        ui.end_row();
        ui.label("Background color");
        ui.color_edit_button_srgba(&mut self.options.canvas_settings.background_color);
    }
}
