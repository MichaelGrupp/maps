use std::default;

use eframe::egui;
use serde::{Deserialize, Serialize};

use crate::app::AppState;
use crate::app_impl::constants::SPACE;
use crate::app_impl::ui_helpers::section_heading;
use crate::render_options::TextureFilter;
use crate::value_colormap::ColorMap;
use crate::value_interpretation::{Mode, Quirks, ValueInterpretation};

use crate::texture_request::NO_TINT;

#[derive(Debug, Serialize, Deserialize)]
pub struct TintOptions {
    #[serde(skip)]
    pub active_tint_selection: Option<String>,
    pub tint_for_all: egui::Color32,
    pub edit_color_to_alpha: bool,
    pub color_to_alpha_for_all: Option<egui::Color32>,
    #[serde(default, skip)]
    pub use_value_interpretation_for_all: bool,
    #[serde(default, skip)]
    pub value_interpretation_for_all: ValueInterpretation,
    #[serde(default, skip)]
    pub colormap_for_all: ColorMap,
    #[serde(default)]
    pub texture_filter_for_all: TextureFilter,
}

impl default::Default for TintOptions {
    fn default() -> Self {
        Self {
            active_tint_selection: None,
            tint_for_all: NO_TINT,
            edit_color_to_alpha: false,
            color_to_alpha_for_all: None,
            use_value_interpretation_for_all: false,
            value_interpretation_for_all: ValueInterpretation::default(),
            colormap_for_all: ColorMap::default(),
            texture_filter_for_all: TextureFilter::default(),
        }
    }
}

impl AppState {
    pub(crate) fn tint_settings(&mut self, ui: &mut egui::Ui) {
        if !section_heading(ui, "Blend", &mut self.options.collapsed.tint_settings) {
            return;
        }
        ui.add_space(SPACE);
        ui.end_row();

        let all_key = "< All >".to_string();
        let selected = self
            .options
            .tint_settings
            .active_tint_selection
            .get_or_insert(all_key.clone());

        // ComboBox is in a horizontal scroll to not take too much space for long paths.
        // Waiting for: https://github.com/emilk/egui/discussions/1829
        egui::ScrollArea::horizontal().show(ui, |ui| {
            egui::ComboBox::from_label("")
                .selected_text(selected.to_string())
                .show_ui(ui, |ui| {
                    ui.selectable_value(selected, all_key.clone(), &all_key);
                    for name in self.data.maps.keys() {
                        ui.selectable_value(selected, name.to_string(), name);
                    }
                });
        });

        let reset = ui.button("Reset").clicked();
        ui.end_row();

        // TODO: clean code below up a bit.
        if reset {
            self.options.tint_settings.edit_color_to_alpha = false;
            self.options.tint_settings.use_value_interpretation_for_all = false;
            self.options.tint_settings.value_interpretation_for_all =
                ValueInterpretation::default();
            self.options.tint_settings.texture_filter_for_all = TextureFilter::default();
        }

        if *selected == all_key {
            let tint = &mut self.options.tint_settings.tint_for_all;
            let color_to_alpha = &mut self.options.tint_settings.color_to_alpha_for_all;
            let value_interpretation = &mut self.options.tint_settings.value_interpretation_for_all;
            let texture_filter = &mut self.options.tint_settings.texture_filter_for_all;

            pick(
                ui,
                reset,
                tint,
                color_to_alpha,
                &mut self.options.tint_settings.edit_color_to_alpha,
                &mut self.options.tint_settings.use_value_interpretation_for_all,
                value_interpretation,
                texture_filter,
            );

            if reset {
                *value_interpretation = ValueInterpretation::default();
                self.options.tint_settings.use_value_interpretation_for_all = false;
            }

            for map in self.data.maps.values_mut() {
                map.tint = Some(*tint);
                map.color_to_alpha = *color_to_alpha;
                map.texture_filter = *texture_filter;
                if self.options.tint_settings.use_value_interpretation_for_all {
                    map.use_value_interpretation = true;
                    map.meta.value_interpretation = *value_interpretation;
                } else {
                    map.meta.reset_value_interpretation();
                    map.use_value_interpretation = false;
                }
            }
        } else if let Some(map) = self.data.maps.get_mut(selected) {
            let tint = map.tint.get_or_insert(NO_TINT);
            let color_to_alpha = &mut map.color_to_alpha;

            if reset {
                map.meta.reset_value_interpretation();
                // If the map has an explicit value interpretation, enable it by default.
                map.use_value_interpretation = map.meta.value_interpretation.explicit_mode;
            }
            pick(
                ui,
                reset,
                tint,
                color_to_alpha,
                &mut self.options.tint_settings.edit_color_to_alpha,
                &mut map.use_value_interpretation,
                &mut map.meta.value_interpretation,
                &mut map.texture_filter,
            )
        } else {
            self.options.tint_settings.active_tint_selection = None;
        }
    }
}

fn pick(
    ui: &mut egui::Ui,
    reset: bool,
    tint: &mut egui::Color32,
    color_to_alpha: &mut Option<egui::Color32>,
    edit_color_to_alpha: &mut bool,
    edit_value_interpretation: &mut bool,
    value_interpretation: &mut ValueInterpretation,
    texture_filter: &mut TextureFilter,
) {
    if reset {
        *tint = NO_TINT;
        *color_to_alpha = None;
        *texture_filter = TextureFilter::default();
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
    ui.end_row();

    pick_filter(ui, texture_filter);
    ui.end_row();

    ui.label("Use value interpretation").on_hover_text(
        "Enable to change the way pixel values are interpreted / thresholded.\n\
        This is enabled by default for maps that have the optional 'mode' parameter set.\n\
        If disabled, the map will be displayed as raw pixel values.",
    );
    ui.checkbox(edit_value_interpretation, "");
    if *edit_value_interpretation {
        ui.end_row();
        ui.end_row();
        pick_value_interpretation(ui, value_interpretation);
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
    ui.label("Tint color / alpha").on_hover_text(
        "Colorize the image with this color.\n\
        Alpha value will be used as transparency.",
    );
    ui.color_edit_button_srgba(tint);
}

fn pick_quirks(ui: &mut egui::Ui, quirks: &mut Quirks) {
    ui.label("Implementation quirks").on_hover_text(
        "Mimic ROS implementation quirks. Choose whether to follow the ROS Wiki\n\
        or what's implemented in ROS' map_server.",
    );
    ui.horizontal(|ui| {
        ui.selectable_value(quirks, Quirks::Ros1Wiki, "ROS 1 Wiki")
            .on_hover_text("Interpret values as documented in ROS 1 Wiki.");
        ui.selectable_value(quirks, Quirks::Ros1MapServer, "ROS map_server")
            .on_hover_text("ROS 1/2 map_server behaves slightly differently than the Wiki :(");
        // ROS 2 is left out because I assume it behaves like ROS 1 map_server.
    });
}

fn pick_mode(ui: &mut egui::Ui, mode: &mut Mode) {
    ui.label("Mode").on_hover_text(
        "How to interpret the pixel values.\n\
         Interpreted pixels are colored in a subsequent step.",
    );
    ui.horizontal(|ui| {
        ui.selectable_value(mode, Mode::Raw, "Raw")
            .on_hover_text("No interpretation, just use the raw pixel values.");
        ui.selectable_value(mode, Mode::Trinary, "Trinary")
            .on_hover_text("Threshold pixel values as free, occupied, or unknown.");
        ui.selectable_value(mode, Mode::Scale, "Scale")
            .on_hover_text(
                "Scale pixel values to continuous range between free and occupied \n\
                 and map pixels with alpha to unknown.",
            );
    });
}

fn pick_colormap(ui: &mut egui::Ui, colormap: &mut ColorMap) {
    ui.label("Coloring")
        .on_hover_text("Select a colormap for the visualization of interpreted pixels.");
    egui::ComboBox::from_label("")
        .selected_text(colormap.to_string())
        .show_ui(ui, |ui| {
            ui.selectable_value(colormap, ColorMap::RvizMap, "RViz \"Map\"")
                .on_hover_text("Classic RViz map coloring.");
            ui.selectable_value(colormap, ColorMap::RvizCostmap, "RViz \"Costmap\"")
                .on_hover_text("Classic RViz costmap coloring.");
            ui.selectable_value(colormap, ColorMap::Raw, "Raw")
                .on_hover_text("No coloring.");
            ui.selectable_value(colormap, ColorMap::CoolCostmap, "Cool costmap")
                .on_hover_text("Alternative costmap coloring with less screaming colors.");
        });
}

fn pick_value_interpretation(ui: &mut egui::Ui, value_interpretation: &mut ValueInterpretation) {
    pick_mode(ui, &mut value_interpretation.mode);
    ui.end_row();
    ui.label("Free threshold")
        .on_hover_text("Threshold for free space interpretation.");
    ui.add(egui::Slider::new(&mut value_interpretation.free, 0.0..=1.0));
    ui.end_row();
    ui.label("Occupied threshold")
        .on_hover_text("Threshold for occupied space interpretation.");
    ui.add(egui::Slider::new(
        &mut value_interpretation.occupied,
        0.0..=1.0,
    ));
    ui.end_row();
    ui.label("Negate")
        .on_hover_text("Negate the pixel interpretation.");
    ui.checkbox(&mut value_interpretation.negate, "");
    ui.end_row();
    pick_colormap(ui, &mut value_interpretation.colormap);
    ui.end_row();
    pick_quirks(ui, &mut value_interpretation.quirks);
}

fn pick_filter(ui: &mut egui::Ui, texture_filter: &mut TextureFilter) {
    ui.label("Texture filter")
        .on_hover_text("How the image texture shall be filtered when rendering.");
    ui.horizontal(|ui| {
        ui.selectable_value(texture_filter, TextureFilter::Smooth, "Smooth")
            .on_hover_text("Linearly interpolate texels for smooth antialiased visualization.");
        ui.selectable_value(texture_filter, TextureFilter::Crisp, "Crisp")
            .on_hover_text("Show texels as sharp squares to see the grid map cells.");
    });
}
