use std::default;

use eframe::egui;
use serde::{Deserialize, Serialize};

use crate::movable::{Draggable, MovableAmounts};

/// Dimension in which grid lines are displayed.
#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum GridLineDimension {
    /// Grid lines are displayed with a constant spacing in screen space.
    Screen,
    #[default]
    /// Grid lines are displayed with a constant spacing in metric space,
    /// i.e. scaling with the grid scale.
    Metric,
}

/// Display type of a grid line.
#[derive(Debug, Default, PartialEq)]
pub enum LineType {
    #[default]
    Main,
    /// Sub grid line between main lines.
    Sub,
}

/// When to show sub grid lines?
#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub enum SubLineVisibility {
    #[default]
    OnlyLens,
    Always,
    Never,
}

/// Which origin markers shall be displayed?
#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub enum MarkerVisibility {
    #[default]
    Zero,
    Map,
    All,
    None,
}

impl MarkerVisibility {
    pub fn any_visible(&self) -> bool {
        matches!(
            self,
            MarkerVisibility::All | MarkerVisibility::Map | MarkerVisibility::Zero
        )
    }

    pub fn zero_visible(&self) -> bool {
        matches!(self, MarkerVisibility::Zero | MarkerVisibility::All)
    }

    pub fn maps_visible(&self) -> bool {
        matches!(self, MarkerVisibility::Map | MarkerVisibility::All)
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(self, MarkerVisibility::Zero, "Zero")
                .on_hover_text("Show marker at the zero coordinate.");
            ui.selectable_value(self, MarkerVisibility::Map, "Map")
                .on_hover_text("Show marker at the origin of each map.");
            ui.selectable_value(self, MarkerVisibility::All, "All")
                .on_hover_text("Show all markers.");
            ui.selectable_value(self, MarkerVisibility::None, "None")
                .on_hover_text("Show no markers.");
        });
    }
}

// Visualization options for the grid that are viewport-independent.
#[derive(Debug, Serialize, Deserialize)]
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
    pub sub_lines_visible: SubLineVisibility,
    pub sub_lines_factor: i8,
    pub sub_lines_stroke: egui::Stroke,
    pub scroll_delta_percent: f32,
    #[serde(default)]
    pub marker_visibility: MarkerVisibility,
    pub marker_length_meters: f32,
    pub marker_width_meters: f32,
    pub marker_x_color: egui::Color32,
    pub marker_y_color: egui::Color32,
    pub marker_z_color: egui::Color32,
    pub tick_labels_visible: bool,
    pub tick_labels_color: egui::Color32,
    #[serde(skip)]
    pub measure_start: Option<egui::Pos2>, // metric
    #[serde(skip)]
    pub measure_end: Option<egui::Pos2>, // metric
    pub measure_stroke: egui::Stroke,
    pub lens_magnification: f32,
    #[serde(skip)]
    pub movable_amounts: MovableAmounts,
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
            sub_lines_visible: SubLineVisibility::default(),
            sub_lines_factor: 10,
            sub_lines_stroke: egui::Stroke::new(
                0.5,
                egui::Color32::from_rgba_unmultiplied(200, 200, 200, 75),
            ),
            scroll_delta_percent: 1.,
            marker_visibility: MarkerVisibility::default(),
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
            movable_amounts: MovableAmounts::default(),
        }
    }
}

impl GridOptions {
    /// Changes the grid scale according to a relative delta in percent.
    /// If `target_pos` is given, the zoom will keep that point at the same screen position.
    /// Otherwise, the zoom will just proportionally scale the offset (viewport-centered zoom).
    /// `target_pos` has to be a screen space position relative to the viewport center.
    pub fn zoom(&mut self, delta_percent: f32, target_pos: Option<egui::Pos2>) {
        let old_scale = self.scale;
        self.scale += delta_percent * 0.01 * self.scale;
        self.scale = self.scale.clamp(self.min_scale, self.max_scale);
        let scale_factor = self.scale / old_scale;
        if let Some(target_pos) = target_pos {
            // Keep the target point at the same screen position.
            let offset_delta = (target_pos - self.offset) * (1. - scale_factor);
            self.offset += offset_delta.to_vec2();
        } else {
            // Viewport-centered zoom.
            self.offset *= scale_factor;
        }
    }
}

impl Draggable for GridOptions {
    fn offset_rhs(&self) -> egui::Vec2 {
        self.offset * egui::vec2(1., -1.) // LHS to RHS
    }

    // Assumes that drag delta is in meters.
    fn drag(&mut self, delta: egui::Vec2) {
        self.offset += delta * self.scale;
    }
}
