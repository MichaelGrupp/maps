use std::collections::HashMap;

use eframe::egui;

use crate::grid_options::{GridLineDimension, GridOptions};
use crate::map_state::MapState;
use crate::texture_request::{RotatedCropRequest, TextureRequest};

pub struct Grid {
    pub metric_extent: egui::Vec2,
    pub points_per_meter: f32,
    pub pixels_per_point: f32,
    pub origin_in_points: egui::Pos2, // Location of the origin in point coordinates.
    pub left_offset: egui::Vec2,
}

// Relations of a RHS metric coordinate map to the LHS point coordinate grid.
struct GridMapRelation {
    scaled_size: egui::Vec2,
    ulc_to_origin_in_points: egui::Vec2, // Upper left corner to map origin in points.
}

impl GridMapRelation {
    fn new(grid: &Grid, map: &MapState) -> GridMapRelation {
        let pixels_per_meter = 1. / map.meta.resolution as f32;
        let points_per_meter = pixels_per_meter * grid.pixels_per_point;
        let scale_factor = grid.points_per_meter / points_per_meter;

        let original_size = egui::Vec2::new(
            map.texture_state.image_pyramid.original.width() as f32,
            map.texture_state.image_pyramid.original.height() as f32,
        );
        let scaled_size = original_size * scale_factor;

        // Meta origin is lower left corner of image in ROS.
        let llc_to_origin_in_points = egui::Vec2::new(
            map.meta.origin.translation.x as f32,
            -map.meta.origin.translation.y as f32, // RHS to LHS
        ) * points_per_meter
            * scale_factor;
        let mut ulc_to_origin_in_points =
            llc_to_origin_in_points - egui::Vec2::new(0., scaled_size.y);

        let translation_in_points =
            map.pose.vec2() * egui::vec2(1., -1.) * points_per_meter * scale_factor;
        ulc_to_origin_in_points = translation_in_points + ulc_to_origin_in_points;

        GridMapRelation {
            scaled_size: scaled_size,
            ulc_to_origin_in_points: ulc_to_origin_in_points,
        }
    }
}

impl Grid {
    pub fn new(ui: &egui::Ui, points_per_meter: f32) -> Grid {
        let available_size = ui.available_size();
        let metric_extent = available_size * points_per_meter;
        // TODO: offset is a hack to avoid wrong drawing when a left side menu is expanded.
        let left_offset = egui::vec2(ui.cursor().min.x, 0.);
        Grid {
            metric_extent: metric_extent,
            points_per_meter: points_per_meter,
            pixels_per_point: ui.ctx().zoom_factor() * ui.ctx().pixels_per_point(),
            origin_in_points: (available_size / 2.).to_pos2(),
            left_offset: left_offset,
        }
    }

    pub fn with_origin_offset(mut self, offset: egui::Vec2) -> Self {
        self.origin_in_points += offset;
        self
    }

    pub fn show_map(&self, ui: &mut egui::Ui, map: &mut MapState, name: &str) {
        if !map.visible {
            return;
        }

        let relation = GridMapRelation::new(self, map);

        let rect = egui::Rect::from_min_size(
            self.origin_in_points + relation.ulc_to_origin_in_points,
            relation.scaled_size,
        );

        let uncropped = TextureRequest::new(name.to_string(), rect).with_tint(map.tint);
        let request = RotatedCropRequest::from_visible(
            ui,
            uncropped,
            map.pose.rot2().inverse(), // RHS to LHS
            relation.ulc_to_origin_in_points,
        );
        map.texture_state.crop_and_put(ui, &request);
    }

    pub fn show_maps(&self, ui: &mut egui::Ui, maps: &mut HashMap<String, MapState>) {
        for (name, map) in maps.iter_mut() {
            self.show_map(ui, map, name);
        }
    }

    fn draw_vertical_lines(
        &self,
        ui: &mut egui::Ui,
        options: &GridOptions,
        spacing_points: f32,
        label_font_id: egui::FontId,
        label_offset: egui::Vec2,
    ) {
        let mut x = self.origin_in_points.x;
        while x > 0. {
            self.draw_vertical_line(ui, x, options, label_font_id.clone(), label_offset);
            x -= spacing_points;
        }
        x = self.origin_in_points.x + spacing_points;
        while x < self.metric_extent.x + self.left_offset.x {
            self.draw_vertical_line(ui, x, options, label_font_id.clone(), label_offset);
            x += spacing_points;
        }
    }

    fn draw_vertical_line(
        &self,
        ui: &mut egui::Ui,
        x: f32,
        options: &GridOptions,
        label_font_id: egui::FontId,
        label_offset: egui::Vec2,
    ) {
        let bottom = egui::Pos2::new(x, self.metric_extent.y / self.points_per_meter);
        ui.painter().line_segment(
            [
                egui::Pos2::new(x, 0.),
                egui::Pos2::new(x, self.metric_extent.y),
            ],
            options.line_stroke,
        );
        if options.tick_labels_visible {
            ui.painter().text(
                bottom - label_offset,
                egui::Align2::LEFT_CENTER,
                format!(
                    "{:.1}",
                    -(self.origin_in_points.x - x) / self.points_per_meter
                ),
                label_font_id,
                options.tick_labels_color,
            );
        }
    }

    fn draw_horizontal_lines(
        &self,
        ui: &mut egui::Ui,
        options: &GridOptions,
        spacing_points: f32,
        label_font_id: egui::FontId,
        label_offset: egui::Vec2,
    ) {
        let mut y = self.origin_in_points.y;
        while y > 0. {
            self.draw_horizontal_line(ui, y, options, label_font_id.clone(), label_offset);
            y -= spacing_points;
        }
        y = self.origin_in_points.y + spacing_points;
        while y < self.metric_extent.y {
            self.draw_horizontal_line(ui, y, options, label_font_id.clone(), label_offset);
            y += spacing_points;
        }
    }

    fn draw_horizontal_line(
        &self,
        ui: &mut egui::Ui,
        y: f32,
        options: &GridOptions,
        label_font_id: egui::FontId,
        label_offset: egui::Vec2,
    ) {
        let left = egui::Pos2::new(0., y) + self.left_offset;
        ui.painter().line_segment(
            [
                left,
                egui::Pos2::new(self.metric_extent.x * self.points_per_meter, y) + self.left_offset,
            ],
            options.line_stroke,
        );
        if options.tick_labels_visible {
            ui.painter().text(
                left + label_offset,
                egui::Align2::LEFT_CENTER,
                format!(
                    "{:.1}",
                    -(self.origin_in_points.y - y) / self.points_per_meter
                ),
                label_font_id,
                options.tick_labels_color,
            );
        }
    }

    pub fn draw(&self, ui: &mut egui::Ui, options: &GridOptions) {
        let spacing_points = match options.line_dimension {
            GridLineDimension::Screen => options.line_spacing_points,
            GridLineDimension::Metric => options.line_spacing_meters * self.points_per_meter,
        };

        let label_font_size = (spacing_points / 3.).min(15.);
        let label_offset = egui::vec2(0., label_font_size / 2.);
        let label_font_id = egui::FontId::new(label_font_size, egui::FontFamily::Monospace);

        self.draw_vertical_lines(
            ui,
            options,
            spacing_points,
            label_font_id.clone(),
            label_offset,
        );
        self.draw_horizontal_lines(ui, options, spacing_points, label_font_id, label_offset);
    }

    pub fn draw_axes(&self, ui: &mut egui::Ui, options: &GridOptions) {
        // Convert stroke width to points.
        let x_stroke = egui::Stroke::new(
            options.marker_width_meters * self.points_per_meter,
            options.marker_x_color,
        );
        let y_stroke = egui::Stroke::new(
            options.marker_width_meters * self.points_per_meter,
            options.marker_y_color,
        );

        ui.painter().line_segment(
            [
                self.origin_in_points,
                self.origin_in_points
                    + egui::vec2(options.marker_length_meters * self.points_per_meter, 0.),
            ],
            x_stroke,
        );
        ui.painter().line_segment(
            [
                self.origin_in_points,
                self.origin_in_points
                    - egui::vec2(0., options.marker_length_meters * self.points_per_meter), // RHS to LHS
            ],
            y_stroke,
        );
        ui.painter().circle_filled(
            self.origin_in_points,
            options.marker_width_meters * self.points_per_meter / 2.,
            options.marker_z_color,
        );
    }
}
