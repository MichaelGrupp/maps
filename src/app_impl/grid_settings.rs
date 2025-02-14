use eframe::egui;

use crate::app::AppState;
use crate::app_impl::constants::SPACE;
use crate::grid_options::{GridLineDimension, GridOptions, SubLineVisibility};

impl AppState {
    pub fn grid_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("Grid");
        if ui.button("Reset").clicked() {
            self.options.grid = GridOptions::default();
        }
        ui.add_space(SPACE);
        ui.end_row();
        ui.label("Show grid lines");
        ui.checkbox(&mut self.options.grid.lines_visible, "");
        ui.end_row();
        ui.label("Show sub grid lines")
            .on_hover_text("Show sub lines between main grid lines.");
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut self.options.grid.sub_lines_visible,
                SubLineVisibility::OnlyLens,
                "In Lens",
            );
            ui.selectable_value(
                &mut self.options.grid.sub_lines_visible,
                SubLineVisibility::Always,
                "Always",
            );
            ui.selectable_value(
                &mut self.options.grid.sub_lines_visible,
                SubLineVisibility::Never,
                "Never",
            );
        });
        ui.end_row();
        ui.label("Grid color");
        ui.horizontal(|ui| {
            ui.color_edit_button_srgba(&mut self.options.grid.line_stroke.color);
            if self.options.grid.sub_lines_visible != SubLineVisibility::Never {
                ui.color_edit_button_srgba(&mut self.options.grid.sub_lines_stroke.color);
            }
        });
        ui.end_row();
        ui.label("Grid spacing dimension");
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut self.options.grid.line_dimension,
                GridLineDimension::Screen,
                "Screen",
            );
            ui.selectable_value(
                &mut self.options.grid.line_dimension,
                GridLineDimension::Metric,
                "Metric",
            );
        });
        ui.end_row();
        match self.options.grid.line_dimension {
            GridLineDimension::Screen => {
                ui.label("Grid lines spacing (points)");
                ui.add(egui::Slider::new(
                    &mut self.options.grid.line_spacing_points,
                    self.options.grid.min_line_spacing_points
                        ..=self.options.grid.max_line_spacing_points,
                ));
            }
            GridLineDimension::Metric => {
                ui.label("Grid lines spacing (meters)");
                ui.add(egui::Slider::new(
                    &mut self.options.grid.line_spacing_meters,
                    self.options.grid.min_line_spacing_meters
                        ..=self.options.grid.max_line_spacing_meters,
                ));
            }
        }
        if self.options.grid.sub_lines_visible != SubLineVisibility::Never {
            ui.end_row();
            ui.label("Sub grid lines factor").on_hover_text(
                "The multiplier for sub lines between main grid lines.\n\
                1 means no sub lines, 2 means one sub line between main lines, etc.",
            );
            ui.add(egui::Slider::new(
                &mut self.options.grid.sub_lines_factor,
                1..=10,
            ));
        }
        ui.end_row();
        ui.end_row();
        ui.label("Show tick labels");
        ui.checkbox(&mut self.options.grid.tick_labels_visible, "");
        ui.end_row();
        ui.label("Tick label color");
        ui.color_edit_button_srgba(&mut self.options.grid.tick_labels_color);
        ui.end_row();
        ui.end_row();
        ui.label("Show marker");
        self.options.grid.marker_visibility.ui(ui);
        ui.end_row();
        if self.options.grid.marker_visibility.any_visible() {
            ui.label("Marker length (meters)");
            ui.add(egui::Slider::new(
                &mut self.options.grid.marker_length_meters,
                0.1..=25.0,
            ));
            ui.end_row();
            ui.label("Marker width (meters)");
            ui.add(egui::Slider::new(
                &mut self.options.grid.marker_width_meters,
                0.01..=5.,
            ));
            ui.end_row();
            ui.label("Marker color (x, y, z)");
            ui.horizontal(|ui| {
                ui.color_edit_button_srgba(&mut self.options.grid.marker_x_color);
                ui.color_edit_button_srgba(&mut self.options.grid.marker_y_color);
                ui.color_edit_button_srgba(&mut self.options.grid.marker_z_color);
            });
            ui.end_row();
        }
        ui.end_row();
        ui.label("Grid scale (points per meter)");
        ui.add(egui::Slider::new(
            &mut self.options.grid.scale,
            self.options.grid.min_scale..=self.options.grid.max_scale,
        ));
        ui.end_row();
        ui.label("Zoom delta (%)")
            .on_hover_text("How much the grid zooms in/out when scrolling.");
        ui.add(egui::Slider::new(
            &mut self.options.grid.scroll_delta_percent,
            0.01..=10.,
        ));
        ui.end_row();
        ui.end_row();

        ui.heading("Tools");
        ui.add_space(SPACE);
        ui.end_row();
        ui.label("Measurement color")
            .on_hover_text("Line color of the measurement tool.");
        ui.color_edit_button_srgba(&mut self.options.grid.measure_stroke.color);
        ui.end_row();
        ui.label("Lens magnification")
            .on_hover_text("Magnification factor for hovering / fixed lenses.");
        ui.add(egui::Slider::new(
            &mut self.options.grid.lens_magnification,
            0.1..=10.0,
        ));
    }
}
