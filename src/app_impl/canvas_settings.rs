use eframe::egui;
use serde::{Deserialize, Serialize};

use crate::app::{AppState, ViewMode};
use crate::app_impl::constants::SPACE;
use crate::app_impl::ui_helpers::section_heading;

const MIN_STACK_SCALE: f32 = 1.0;
const MAX_STACK_SCALE: f32 = 10.0;

const fn default_stack_scale_factor() -> f32 {
    MIN_STACK_SCALE
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CanvasOptions {
    pub background_color: egui::Color32,
    #[serde(default)]
    pub theme_preference: egui::ThemePreference,
    #[serde(skip, default = "default_stack_scale_factor")]
    pub stack_scale_factor: f32,
}

impl Default for CanvasOptions {
    fn default() -> Self {
        Self {
            background_color: egui::Visuals::default().faint_bg_color,
            theme_preference: egui::ThemePreference::default(),
            stack_scale_factor: MIN_STACK_SCALE,
        }
    }
}

impl AppState {
    pub(crate) fn canvas_settings(&mut self, ui: &mut egui::Ui) {
        section_heading(ui, "Canvas", &mut self.options.collapsed.canvas_settings);
        if ui.button("Reset").clicked() {
            self.options.canvas_settings = CanvasOptions::default();
        }
        if self.options.collapsed.canvas_settings {
            return;
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

        if self.options.view_mode == ViewMode::Stacked {
            ui.end_row();
            ui.label("Stack scale factor").on_hover_text(
                "Scale factor for the stacked view. \
                 1.0 fits all maps into the canvas.",
            );
            ui.add(egui::Slider::new(
                &mut self.options.canvas_settings.stack_scale_factor,
                MIN_STACK_SCALE..=MAX_STACK_SCALE,
            ));
        }
    }
}
