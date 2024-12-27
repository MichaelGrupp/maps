use std::default;

use eframe::egui;

// Visualization options for the grid that are viewport-independent.
#[derive(Debug)]
pub struct GridOptions {
    pub scale: f32,
    pub min_scale: f32,
    pub max_scale: f32,
    pub offset: egui::Vec2,
    pub lines_visible: bool,
    pub line_spacing: f32,
    pub min_line_spacing: f32,
    pub max_line_spacing: f32,
    pub line_stroke: egui::Stroke,
    pub scroll_speed_factor: f32,
}

impl default::Default for GridOptions {
    fn default() -> Self {
        GridOptions {
            scale: 5.,
            min_scale: 1.,
            max_scale: 100.,
            offset: egui::Vec2::new(0., 0.),
            lines_visible: false,
            line_spacing: 1.,
            min_line_spacing: 0.1,
            max_line_spacing: 100.,
            line_stroke: egui::Stroke::new(1., egui::Color32::LIGHT_BLUE),
            scroll_speed_factor: 0.2,
        }
    }
}
