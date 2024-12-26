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

    pub fn with_origin_offset(&mut self, offset: egui::Vec2) -> &Self {
        self.origin_in_points += offset;
        self
    }

    pub fn show_map(&self, ui: &mut egui::Ui, map: &mut MapState, name: &str) {
        if !map.visible {
            return;
        }

        let relation = GridMapRelation::new(self, map);

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
        let texture = map.texture_state.texture_handle.as_ref().unwrap();
        ui.put(rect, egui::Image::new(texture));
    }

    pub fn show_maps(&self, ui: &mut egui::Ui, maps: &mut HashMap<String, MapState>) {
        for (name, map) in maps.iter_mut() {
            self.show_map(ui, map, name);
        }
    }
}
