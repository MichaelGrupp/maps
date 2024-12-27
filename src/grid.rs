use std::collections::HashMap;

use eframe::egui;

use crate::map_state::MapState;

pub struct Grid {
    pub metric_extent: egui::Vec2,
    pub points_per_meter: f32,
    pub pixels_per_point: f32,
    pub origin_in_points: egui::Pos2, // Location of the origin in point coordinates.
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
        let ulc_to_origin_in_points = llc_to_origin_in_points - egui::Vec2::new(0., scaled_size.y);

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
        Grid {
            metric_extent: metric_extent,
            points_per_meter: points_per_meter,
            pixels_per_point: ui.ctx().zoom_factor() * ui.ctx().pixels_per_point(),
            origin_in_points: (available_size / 2.).to_pos2(),
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

        // TODO: Crop to viewport to not explode.
        if map.texture_state.desired_size != relation.scaled_size {
            map.texture_state.texture_handle = None;
            map.texture_state.desired_size = relation.scaled_size;
        }
        map.texture_state
            .update(ui, format!("{} in grid", name).as_str());

        let rect = egui::Rect::from_min_size(
            self.origin_in_points + relation.ulc_to_origin_in_points,
            relation.scaled_size,
        );

        // Calculate the intersection of the image and the viewport.
        let viewport_rect = ui.clip_rect();
        let visible_rect = rect.intersect(viewport_rect);

        // Calculate the UV coordinates for the visible portion.
        let uv_min = egui::Pos2::new(
            (visible_rect.min.x - rect.min.x) / rect.width(),
            (visible_rect.min.y - rect.min.y) / rect.height(),
        );
        let uv_max = egui::Pos2::new(
            (visible_rect.max.x - rect.min.x) / rect.width(),
            (visible_rect.max.y - rect.min.y) / rect.height(),
        );

        let texture = map.texture_state.texture_handle.as_ref().unwrap();
        map.texture_state.image_response = Some(
            ui.put(
                visible_rect,
                egui::Image::new(texture)
                    .maintain_aspect_ratio(false)
                    .uv([uv_min, uv_max])
                    .fit_to_exact_size(visible_rect.size()),
            ),
        );
    }

    pub fn show_maps(&self, ui: &mut egui::Ui, maps: &mut HashMap<String, MapState>) {
        for (name, map) in maps.iter_mut() {
            self.show_map(ui, map, name);
        }
    }

    pub fn draw(&self, ui: &mut egui::Ui, spacing_meters: f32, stroke: egui::Stroke) {
        // Draw grid lines in whole area, but always cross the origin.
        // Start from the origin and go to left and right.
        let spacing_points = spacing_meters * self.points_per_meter;
        let mut x = self.origin_in_points.x;
        while x > 0. {
            ui.painter().line_segment(
                [
                    egui::Pos2::new(x, 0.),
                    egui::Pos2::new(x, self.metric_extent.y),
                ],
                stroke,
            );
            x -= spacing_points;
        }
        x = self.origin_in_points.x + spacing_points;
        while x < self.metric_extent.x {
            ui.painter().line_segment(
                [
                    egui::Pos2::new(x, 0.),
                    egui::Pos2::new(x, self.metric_extent.y),
                ],
                stroke,
            );
            x += spacing_points;
        }

        // Now for the vertical lines.
        let mut y = self.origin_in_points.y;
        while y > 0. {
            ui.painter().line_segment(
                [
                    egui::Pos2::new(0., y),
                    egui::Pos2::new(self.metric_extent.x, y),
                ],
                stroke,
            );
            y -= spacing_points;
        }
        y = self.origin_in_points.y + spacing_points;
        while y < self.metric_extent.y {
            ui.painter().line_segment(
                [
                    egui::Pos2::new(0., y),
                    egui::Pos2::new(self.metric_extent.x, y),
                ],
                stroke,
            );
            y += spacing_points;
        }
    }
}
