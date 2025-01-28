use std::default;

use eframe::egui;
use serde::{Deserialize, Serialize};

use crate::app::AppState;
use crate::app_impl::constants::SPACE;

use crate::texture_request::NO_TINT;

#[derive(Debug, Serialize, Deserialize)]
pub struct TintOptions {
    #[serde(skip)]
    pub active_tint_selection: Option<String>,
    pub tint_for_all: egui::Color32,
    pub edit_color_to_alpha: bool,
    pub color_to_alpha_for_all: Option<egui::Color32>,
}

impl default::Default for TintOptions {
    fn default() -> Self {
        Self {
            active_tint_selection: None,
            tint_for_all: NO_TINT,
            edit_color_to_alpha: false,
            color_to_alpha_for_all: None,
        }
    }
}

impl AppState {
    pub fn tint_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("Blend");
        ui.add_space(SPACE);
        ui.end_row();

        let all_key = "< All >".to_string();
        let selected = self
            .options
            .tint_settings
            .active_tint_selection
            .get_or_insert(all_key.clone());
        egui::ComboBox::from_label("")
            .selected_text(selected.to_string())
            .show_ui(ui, |ui| {
                ui.selectable_value(selected, all_key.clone(), &all_key);
                for name in self.data.maps.keys() {
                    ui.selectable_value(selected, name.to_string(), name);
                }
            });

        let reset = ui.button("Reset").clicked();
        ui.end_row();

        if reset {
            self.options.tint_settings.edit_color_to_alpha = false;
        }

        if *selected == all_key {
            let tint = &mut self.options.tint_settings.tint_for_all;
            let color_to_alpha = &mut self.options.tint_settings.color_to_alpha_for_all;

            pick(
                ui,
                reset,
                tint,
                color_to_alpha,
                &mut self.options.tint_settings.edit_color_to_alpha,
            );

            for map in self.data.maps.values_mut() {
                map.tint = Some(*tint);
                map.color_to_alpha = *color_to_alpha;
            }
        } else if let Some(map) = self.data.maps.get_mut(selected) {
            let tint = map.tint.get_or_insert(NO_TINT);
            let color_to_alpha = &mut map.color_to_alpha;

            pick(
                ui,
                reset,
                tint,
                color_to_alpha,
                &mut self.options.tint_settings.edit_color_to_alpha,
            );
        } else {
            self.options.tint_settings.active_tint_selection = None;
        }
    }
}

fn pick_color_to_alpha(ui: &mut egui::Ui, color_to_alpha: &mut Option<egui::Color32>) {
    ui.label("Color for alpha mapping").on_hover_text(
        "Select a pixel value (of the source image) that shall be shown as transparent.",
    );
    if let Some(color_to_alpha) = color_to_alpha {
        ui.color_edit_button_srgba(color_to_alpha);
    } else {
        *color_to_alpha = Some(egui::Color32::from_gray(128));
    }
}

fn pick_tint_color(ui: &mut egui::Ui, tint: &mut egui::Color32) {
    ui.label("Tint color");
    ui.color_edit_button_srgba(tint);
}

fn pick(
    ui: &mut egui::Ui,
    reset: bool,
    tint: &mut egui::Color32,
    color_to_alpha: &mut Option<egui::Color32>,
    edit_color_to_alpha: &mut bool,
) {
    if reset {
        *tint = NO_TINT;
        *color_to_alpha = None;
    }

    pick_tint_color(ui, tint);
    ui.end_row();

    ui.label("Enable color to alpha")
        .on_hover_text("Enable to select a pixel value that shall be shown as transparent.");
    ui.checkbox(edit_color_to_alpha, "");
    if *edit_color_to_alpha {
        ui.end_row();
        pick_color_to_alpha(ui, color_to_alpha);
    } else {
        *color_to_alpha = None;
    }
}
