use eframe::egui;

use crate::app::AppState;
use crate::app_impl::constants::SPACE;
use crate::grid_options::GridOptions;

impl AppState {
    pub fn grid_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("Grid");
        if ui.button("Reset").clicked() {
            self.options.grid = GridOptions::default();
        }
        ui.add_space(SPACE);
        ui.end_row();
        ui.label("Show Grid Lines");
        ui.checkbox(&mut self.options.grid.lines_visible, "");
        ui.end_row();
        ui.label("Grid color");
        ui.color_edit_button_srgba(&mut self.options.grid.line_stroke.color);
        ui.end_row();
        ui.label("Grid lines spacing (meters)");
        ui.add(egui::Slider::new(
            &mut self.options.grid.line_spacing_meters,
            self.options.grid.min_line_spacing..=self.options.grid.max_line_spacing,
        ));
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
        ui.checkbox(&mut self.options.grid.marker_visible, "");
        ui.end_row();
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
        ui.label("Marker color (X)");
        ui.color_edit_button_srgba(&mut self.options.grid.marker_x_color);
        ui.end_row();
        ui.label("Marker color (Y)");
        ui.color_edit_button_srgba(&mut self.options.grid.marker_y_color);
        ui.end_row();
        ui.label("Marker color (Z)");
        ui.color_edit_button_srgba(&mut self.options.grid.marker_z_color);
        ui.end_row();
        ui.end_row();
        ui.label("Grid scale (points per meter)");
        ui.add(egui::Slider::new(
            &mut self.options.grid.scale,
            self.options.grid.min_scale..=self.options.grid.max_scale,
        ));
        ui.end_row();
        ui.label("Zoom speed")
            .on_hover_text("How fast the grid zooms in/out when scrolling.");
        ui.add(egui::Slider::new(
            &mut self.options.grid.scroll_speed_factor,
            0.0..=1.0,
        ));
    }
}
