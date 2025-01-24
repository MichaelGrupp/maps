use std::default;

use eframe::egui;

use crate::movable::Draggable;

#[derive(Debug, Default, PartialEq)]
pub enum GridLineDimension {
    Screen,
    #[default]
    Metric,
}

// Visualization options for the grid that are viewport-independent.
#[derive(Debug)]
pub struct GridOptions {
    pub scale: f32,
    pub min_scale: f32,
    pub max_scale: f32,
    pub offset: egui::Vec2,
    pub lines_visible: bool,
    pub line_dimension: GridLineDimension,
    pub line_spacing_meters: f32,
    pub min_line_spacing_meters: f32,
    pub max_line_spacing_meters: f32,
    pub line_spacing_points: f32,
    pub min_line_spacing_points: f32,
    pub max_line_spacing_points: f32,
    pub line_stroke: egui::Stroke,
    pub scroll_delta_percent: f32,
    pub marker_visible: bool,
    pub marker_length_meters: f32,
    pub marker_width_meters: f32,
    pub marker_x_color: egui::Color32,
    pub marker_y_color: egui::Color32,
    pub marker_z_color: egui::Color32,
    pub tick_labels_visible: bool,
    pub tick_labels_color: egui::Color32,
    pub measure_start: Option<egui::Pos2>, // metric
    pub measure_end: Option<egui::Pos2>,   // metric
    pub measure_stroke: egui::Stroke,
    pub lens_magnification: f32,
}

impl default::Default for GridOptions {
    fn default() -> Self {
        GridOptions {
            line_dimension: GridLineDimension::default(),
            scale: 25.,
            min_scale: 1.,
            max_scale: 500.,
            offset: egui::Vec2::new(0., 0.),
            lines_visible: true,
            line_spacing_meters: 10.,
            min_line_spacing_meters: 0.1,
            max_line_spacing_meters: 100.,
            line_spacing_points: 200.,
            min_line_spacing_points: 1.,
            max_line_spacing_points: 1000.,
            line_stroke: egui::Stroke::new(1., egui::Color32::LIGHT_BLUE),
            scroll_delta_percent: 1.,
            marker_visible: true,
            marker_length_meters: 1.,
            marker_width_meters: 0.1,
            marker_x_color: egui::Color32::RED,
            marker_y_color: egui::Color32::GREEN,
            marker_z_color: egui::Color32::BLUE,
            tick_labels_visible: true,
            tick_labels_color: egui::Color32::DARK_GRAY,
            measure_start: None,
            measure_end: None,
            measure_stroke: egui::Stroke::new(2., egui::Color32::ORANGE),
            lens_magnification: 5.,
        }
    }
}

impl GridOptions {
    pub fn zoom(&mut self, delta_percent: f32) {
        // Viewport-centered zoom.
        let old_scale = self.scale;
        self.scale += delta_percent * 0.01 * self.scale;
        self.scale = self.scale.clamp(self.min_scale, self.max_scale);
        let scale_factor = self.scale / old_scale;
        self.offset *= scale_factor;
    }
}

impl Draggable for GridOptions {
    // Assumes that drag delta is in meters.
    fn drag(&mut self, delta: egui::Vec2) {
        self.offset += delta * self.scale;
    }
}
