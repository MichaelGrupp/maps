use std::collections::BTreeMap;

use eframe::egui;

use crate::grid_options::{GridLineDimension, GridOptions, LineType};
use crate::map_pose::MapPose;
use crate::map_state::MapState;
use crate::texture_request::{RotatedCropRequest, TextureRequest};

pub struct Grid {
    pub name: String,
    pub ui_offset: egui::Vec2,
    pub metric_extent: egui::Vec2,
    pub points_per_meter: f32,
    pub origin_in_points: egui::Pos2, // Location of the origin in point coordinates.
    pub left_offset: egui::Vec2,
}

// Relations of a RHS metric coordinate map to the LHS point coordinate grid.
struct GridMapRelation {
    scaled_size: egui::Vec2,
    ulc_to_origin_in_points: egui::Vec2, // Upper left corner to map origin in points.
    ulc_to_origin_in_points_translated: egui::Vec2, // Includes map pose translation.
}

impl GridMapRelation {
    fn new(grid: &Grid, map: &mut MapState, id: &str) -> GridMapRelation {
        let points_per_meter = 1. / map.meta.resolution;
        let scale_factor = grid.points_per_meter / points_per_meter;

        let texture_state = map.get_or_create_texture_state(id);

        let original_size = egui::Vec2::new(
            texture_state.image_pyramid.original.width() as f32,
            texture_state.image_pyramid.original.height() as f32,
        );
        let scaled_size = original_size * scale_factor;

        let rhs_to_lhs = egui::vec2(1., -1.); // Screen is a LHS system.

        // Meta origin is lower left corner of image in ROS.
        let llc_to_origin_in_points =
            map.meta.origin_xy * rhs_to_lhs * points_per_meter * scale_factor;
        let ulc_to_origin_in_points = llc_to_origin_in_points - egui::Vec2::new(0., scaled_size.y);

        let translation_in_points = map.pose.vec2() * rhs_to_lhs * points_per_meter * scale_factor;
        let ulc_to_origin_in_points_translated = translation_in_points + ulc_to_origin_in_points;

        GridMapRelation {
            scaled_size,
            ulc_to_origin_in_points,
            ulc_to_origin_in_points_translated,
        }
    }
}

struct LabelTextOptions {
    font_id: egui::FontId,
    offset: egui::Vec2,
}

impl Grid {
    pub fn new(ui: &egui::Ui, name: &str, points_per_meter: f32) -> Grid {
        // Where are we with this UI in the global context?
        // Required to offset the origin because we paint at manual positions.
        let ui_offset = ui.clip_rect().min.to_vec2();

        let available_size = ui.available_size();
        let metric_extent = available_size * points_per_meter;
        // TODO: offset is a hack to avoid wrong drawing when a left side menu is expanded.
        let left_offset = egui::vec2(ui.cursor().min.x, 0.);
        Grid {
            name: name.to_string(),
            ui_offset,
            metric_extent,
            points_per_meter,
            origin_in_points: (available_size / 2.).to_pos2() + ui_offset,
            left_offset,
        }
    }

    pub fn with_origin_offset(mut self, offset: egui::Vec2) -> Self {
        self.origin_in_points += offset;
        self
    }

    pub fn centered_at(self, metric_pos: egui::Pos2) -> Self {
        let offset = -metric_pos.to_vec2() * self.points_per_meter;
        self.with_origin_offset(offset)
    }

    pub fn to_metric(&self, point: &egui::Pos2) -> egui::Pos2 {
        ((*point - self.origin_in_points) / self.points_per_meter).to_pos2()
    }

    pub fn to_point(&self, metric: &egui::Pos2) -> egui::Pos2 {
        *metric * self.points_per_meter + self.origin_in_points.to_vec2()
    }

    pub fn show_map(
        &self,
        ui: &mut egui::Ui,
        map: &mut MapState,
        map_name: &str,
        options: &GridOptions,
    ) {
        if !map.visible {
            return;
        }

        let relation = GridMapRelation::new(self, map, self.name.as_str());

        let rect = egui::Rect::from_min_size(
            self.origin_in_points + relation.ulc_to_origin_in_points,
            relation.scaled_size,
        );

        let pose_rotation = map.pose.rot2().inverse(); // RHS to LHS
        let origin_rotation = map.meta.origin_theta.inverse();

        let uncropped = TextureRequest::new(map_name.to_string(), rect)
            .with_tint(map.tint)
            .with_color_to_alpha(map.color_to_alpha)
            .with_thresholding(map.get_value_interpretation());
        let request = RotatedCropRequest::from_visible(
            ui,
            uncropped,
            pose_rotation * origin_rotation,
            relation.ulc_to_origin_in_points_translated - relation.ulc_to_origin_in_points,
            relation.ulc_to_origin_in_points,
        );
        map.get_or_create_texture_state(self.name.as_str())
            .crop_and_put(ui, &request);

        if options.marker_visibility.maps_visible() {
            self.draw_axes(ui, options, Some(&map.pose));
        }
    }

    pub fn show_maps(
        &self,
        ui: &mut egui::Ui,
        maps: &mut BTreeMap<String, MapState>,
        options: &GridOptions,
    ) {
        for (name, map) in maps.iter_mut() {
            self.show_map(ui, map, name, options);
        }
    }

    pub fn hover_pos_metric(&self, ui: &egui::Ui) -> Option<egui::Pos2> {
        if !ui.rect_contains_pointer(ui.clip_rect()) {
            return None;
        }
        ui.ctx().pointer_hover_pos().map(|pos| self.to_metric(&pos))
    }

    fn draw_vertical_lines(
        &self,
        ui: &mut egui::Ui,
        options: &GridOptions,
        line_type: &LineType,
        spacing_points: f32,
        label_text_options: &Option<LabelTextOptions>,
    ) {
        let mut x = self.origin_in_points.x;
        while x > 0. {
            self.draw_vertical_line(ui, x, options, line_type, label_text_options);
            x -= spacing_points;
        }
        x = self.origin_in_points.x + spacing_points;
        while x < ui.available_width() + self.ui_offset.x {
            self.draw_vertical_line(ui, x, options, line_type, label_text_options);
            x += spacing_points;
        }
    }

    fn draw_vertical_line(
        &self,
        ui: &mut egui::Ui,
        x: f32,
        options: &GridOptions,
        line_type: &LineType,
        label_text_options: &Option<LabelTextOptions>,
    ) {
        let bottom = egui::Pos2::new(x, self.metric_extent.y / self.points_per_meter);
        let stroke = match line_type {
            LineType::Main => &options.line_stroke,
            LineType::Sub => &options.sub_lines_stroke,
        };
        ui.painter().line_segment(
            [
                egui::Pos2::new(x, 0.),
                egui::Pos2::new(x, ui.available_height() + self.ui_offset.y),
            ],
            *stroke,
        );
        if !options.tick_labels_visible {
            return;
        }
        if let Some(label_options) = label_text_options {
            ui.painter().text(
                bottom - label_options.offset + egui::vec2(0., self.ui_offset.y),
                egui::Align2::LEFT_CENTER,
                format!(
                    "{:.1}",
                    -(self.origin_in_points.x - x) / self.points_per_meter
                ),
                label_options.font_id.clone(),
                options.tick_labels_color,
            );
        }
    }

    fn draw_horizontal_lines(
        &self,
        ui: &mut egui::Ui,
        options: &GridOptions,
        line_type: &LineType,
        spacing_points: f32,
        label_text_options: &Option<LabelTextOptions>,
    ) {
        let mut y = self.origin_in_points.y;
        while y > 0. {
            self.draw_horizontal_line(ui, y, options, line_type, label_text_options);
            y -= spacing_points;
        }
        y = self.origin_in_points.y + spacing_points;
        while y < ui.available_height() + self.ui_offset.y {
            self.draw_horizontal_line(ui, y, options, line_type, label_text_options);
            y += spacing_points;
        }
    }

    fn draw_horizontal_line(
        &self,
        ui: &mut egui::Ui,
        y: f32,
        options: &GridOptions,
        line_type: &LineType,
        label_text_options: &Option<LabelTextOptions>,
    ) {
        let left = egui::Pos2::new(0., y) + self.left_offset;
        let stroke = match line_type {
            LineType::Main => &options.line_stroke,
            LineType::Sub => &options.sub_lines_stroke,
        };
        ui.painter().line_segment(
            [
                left,
                egui::Pos2::new(ui.available_width() + self.ui_offset.x, y) + self.left_offset,
            ],
            *stroke,
        );
        if !options.tick_labels_visible {
            return;
        }
        if let Some(label_options) = label_text_options {
            ui.painter().text(
                left + label_options.offset,
                egui::Align2::LEFT_CENTER,
                format!(
                    "{:.1}",
                    (self.origin_in_points.y - y) / self.points_per_meter
                ),
                label_options.font_id.clone(),
                options.tick_labels_color,
            );
        }
    }

    pub fn draw(&self, ui: &mut egui::Ui, options: &GridOptions, line_type: LineType) {
        if !options.lines_visible {
            return;
        }

        let mut spacing_points = match options.line_dimension {
            GridLineDimension::Screen => options.line_spacing_points,
            GridLineDimension::Metric => options.line_spacing_meters * self.points_per_meter,
        };
        if line_type == LineType::Sub {
            spacing_points /= options.sub_lines_factor as f32;
        }

        let label_text_options: Option<LabelTextOptions> = match line_type {
            LineType::Main => {
                let label_font_size = (spacing_points / 3.).min(15.);
                Some(LabelTextOptions {
                    font_id: egui::FontId::new(label_font_size, egui::FontFamily::Monospace),
                    offset: egui::vec2(0., label_font_size / 2.),
                })
            }
            LineType::Sub => None,
        };

        self.draw_vertical_lines(ui, options, &line_type, spacing_points, &label_text_options);
        self.draw_horizontal_lines(ui, options, &line_type, spacing_points, &label_text_options);
    }

    pub fn draw_axes(&self, ui: &mut egui::Ui, options: &GridOptions, pose: Option<&MapPose>) {
        // Convert stroke width to points.
        let x_stroke = egui::Stroke::new(
            options.marker_width_meters * self.points_per_meter,
            options.marker_x_color,
        );
        let y_stroke = egui::Stroke::new(
            options.marker_width_meters * self.points_per_meter,
            options.marker_y_color,
        );

        let pos = match pose {
            Some(p) => self.to_point(&(p.vec2() * egui::vec2(1., -1.)).to_pos2()),
            None => self.origin_in_points,
        };
        let x_vec = match pose {
            Some(p) => p.rot2().inverse() * egui::vec2(1., 0.),
            None => egui::vec2(1., 0.),
        } * options.marker_length_meters
            * self.points_per_meter;
        let y_vec = match pose {
            Some(p) => p.rot2().inverse() * egui::vec2(0., 1.),
            None => egui::vec2(0., 1.),
        } * options.marker_length_meters
            * self.points_per_meter;

        ui.painter().line_segment([pos, pos + x_vec], x_stroke);
        ui.painter().line_segment([pos, pos - y_vec], y_stroke);
        ui.painter().circle_filled(
            pos,
            options.marker_width_meters * self.points_per_meter / 2.,
            options.marker_z_color,
        );
    }

    pub fn draw_measure(
        &self,
        ui: &mut egui::Ui,
        options: &GridOptions,
        temporary_end: Option<egui::Pos2>,
    ) {
        if let Some(start_metric) = options.measure_start {
            let start = self.to_point(&start_metric);
            ui.painter().circle_filled(
                start,
                options.measure_stroke.width * 2.,
                options.measure_stroke.color,
            );

            let end_metric = options
                .measure_end
                .unwrap_or(temporary_end.unwrap_or(start_metric));
            let end = self.to_point(&end_metric);
            ui.painter()
                .line_segment([start, end], options.measure_stroke);
            ui.painter().circle_filled(
                end,
                options.measure_stroke.width * 2.,
                options.measure_stroke.color,
            );
            ui.painter().text(
                end,
                egui::Align2::LEFT_BOTTOM,
                format!("{:.3} m", (end_metric - start_metric).length()),
                egui::FontId::new(15., egui::FontFamily::Monospace),
                options.measure_stroke.color,
            );
        }
    }
}
