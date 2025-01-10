use std::default;

use eframe::egui;

#[derive(Debug, Default, PartialEq)]
pub enum GridLineDimension {
    Screen,
    #[default]
    Metric,
}

pub enum DragDirection {
    Up,
    Down,
    Left,
    Right,
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
    pub scroll_speed_factor: f32,
    pub marker_visible: bool,
    pub marker_length_meters: f32,
    pub marker_width_meters: f32,
    pub marker_x_color: egui::Color32,
    pub marker_y_color: egui::Color32,
    pub marker_z_color: egui::Color32,
    pub tick_labels_visible: bool,
    pub tick_labels_color: egui::Color32,
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
            scroll_speed_factor: 0.2,
            marker_visible: true,
            marker_length_meters: 1.,
            marker_width_meters: 0.1,
            marker_x_color: egui::Color32::RED,
            marker_y_color: egui::Color32::GREEN,
            marker_z_color: egui::Color32::BLUE,
            tick_labels_visible: true,
            tick_labels_color: egui::Color32::DARK_GRAY,
        }
    }
}

impl GridOptions {
    pub fn zoom(&mut self, delta: f32) {
        // Viewport-centered zoom.
        let old_scale = self.scale;
        self.scale += delta;
        self.scale = self.scale.clamp(self.min_scale, self.max_scale);
        let scale_factor = self.scale / old_scale;
        self.offset *= scale_factor;
    }

    pub fn drag(&mut self, delta: egui::Vec2) {
        self.offset += delta;
    }

    pub fn drag_directed(&mut self, amount: f32, direction: DragDirection) {
        let delta = match direction {
            DragDirection::Up => egui::vec2(0., -amount),
            DragDirection::Down => egui::vec2(0., amount),
            DragDirection::Left => egui::vec2(-amount, 0.),
            DragDirection::Right => egui::vec2(amount, 0.),
        };
        self.drag(delta);
    }
}
