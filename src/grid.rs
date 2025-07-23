use std::collections::BTreeMap;

use eframe::egui;
use log::error;

use crate::draw_order::DrawOrder;
use crate::grid_options::{GridLineDimension, GridOptions, LineType};
use crate::map_pose::MapPose;
use crate::map_state::MapState;
use crate::movable::Draggable;
use crate::texture_request::{ImagePlacement, RotatedCropRequest, TextureRequest};

/// Grid area for displaying metric objects in screen space (points).
pub struct Grid {
    pub name: String,
    /// Screen space offset of the grid's UI area.
    ui_offset: egui::Vec2,
    /// Display scale of the grid: how many points per meter?
    pub points_per_meter: f32,
    /// Location of the origin in point coordinates.
    pub origin_in_points: egui::Pos2,
    texture_crop_threshold: u32,
    response: egui::Response,
    painter: egui::Painter,
}

/// Relations of a RHS metric coordinate map to the LHS point coordinate grid.
struct GridMapRelation {
    /// Size of the map scaled to the grid's zoom level, in points.
    scaled_size: egui::Vec2,
    /// Points occupied by a cell (pixel) of the map.
    points_per_cell: f32,
    /// Upper left image corner to map origin in points, without map pose translation.
    ulc_to_origin_in_points: egui::Vec2,
    /// Upper left image corner to map origin in points, including map pose translation.
    ulc_to_origin_in_points_translated: egui::Vec2,
}

/// Switches from left- to right-handed coordinate system or vice versa.
/// Screen is a LHS system.
fn flip(vec: egui::Vec2) -> egui::Vec2 {
    vec * egui::vec2(1., -1.)
}

impl GridMapRelation {
    fn new(grid: &Grid, map: &mut MapState) -> GridMapRelation {
        let points_per_meter = 1. / map.meta.resolution;
        let scale_factor = grid.points_per_meter / points_per_meter;
        let points_per_cell = points_per_meter * map.meta.resolution * scale_factor;
        let scaled_size = map.image_pyramid.original_size * scale_factor;

        // Meta origin is lower left corner of image in ROS.
        let llc_to_origin_in_points = flip(map.meta.origin_xy) * points_per_meter * scale_factor;
        let ulc_to_origin_in_points = llc_to_origin_in_points - egui::Vec2::new(0., scaled_size.y);

        let translation_in_points = flip(map.pose.vec2()) * points_per_meter * scale_factor;
        let ulc_to_origin_in_points_translated = translation_in_points + ulc_to_origin_in_points;

        GridMapRelation {
            scaled_size,
            points_per_cell,
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
    /// Creates a new grid to be drawn in the available space of the given `ui`
    /// with the desired scale defined by `points_per_meter`.
    /// Note that the grid is valid only for one frame, so it should not be
    /// persisted across frames. Recreate every frame to adapt to the latest UI.
    pub fn new(ui: &mut egui::Ui, name: &str, points_per_meter: f32) -> Grid {
        let (response, painter) =
            ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
        // Where are we with this UI in the global context?
        // Required to offset the origin because we paint at manual positions.
        let ui_offset = painter.clip_rect().min.to_vec2();
        let paint_rect_size = painter.clip_rect().size();

        Grid {
            name: name.to_string(),
            ui_offset,
            points_per_meter,
            origin_in_points: (paint_rect_size / 2.).to_pos2() + ui_offset,
            texture_crop_threshold: 0,
            response,
            painter,
        }
    }

    /// Sets the position offset of the grid's zero origin, in points.
    pub fn with_origin_offset(mut self, offset: egui::Vec2) -> Self {
        self.origin_in_points += offset;
        self
    }

    /// Fixates the grid to be displayed with the specified metric position in the UI center.
    pub fn centered_at(self, metric_pos: egui::Pos2) -> Self {
        let offset = flip(-metric_pos.to_vec2()) * self.points_per_meter;
        self.with_origin_offset(offset)
    }

    /// Sets the threshold for texture cropping, i.e. the max texture width/height that can
    /// be displayed without cropping.
    /// Set this threshold to allow displaying images larger than the GPU texture buffer.
    pub fn with_texture_crop_threshold(mut self, threshold: u32) -> Self {
        self.texture_crop_threshold = threshold;
        self
    }

    /// Converts a point grid coordinate to a metric coordinate.
    pub fn to_metric(&self, point: &egui::Pos2) -> egui::Pos2 {
        (flip(*point - self.origin_in_points) / self.points_per_meter).to_pos2()
    }

    /// Converts a metric grid coordinate to points.
    pub fn to_point(&self, metric: &egui::Pos2) -> egui::Pos2 {
        flip(metric.to_vec2()).to_pos2() * self.points_per_meter + self.origin_in_points.to_vec2()
    }

    /// Adds a single map to be displayed.
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

        let relation = GridMapRelation::new(self, map);

        let rect = egui::Rect::from_min_size(
            self.origin_in_points + relation.ulc_to_origin_in_points,
            relation.scaled_size,
        );

        let pose_rotation = map.pose.rot2().inverse(); // RHS to LHS
        let origin_rotation = map.meta.origin_theta.inverse();

        let uncropped = TextureRequest::new(map_name.to_string(), rect)
            .with_tint(map.tint)
            .with_color_to_alpha(map.color_to_alpha)
            .with_thresholding(map.get_value_interpretation())
            .with_texture_options(&map.texture_filter.get(relation.points_per_cell));

        let placement = ImagePlacement {
            rotation: pose_rotation * origin_rotation,
            translation: relation.ulc_to_origin_in_points_translated
                - relation.ulc_to_origin_in_points,
            rotation_center: relation.ulc_to_origin_in_points,
            points_per_pixel: relation.points_per_cell,
            original_image_size: map.image_pyramid.original_size,
        };

        let request = RotatedCropRequest::from_visible(
            ui,
            &self.painter.clip_rect(),
            uncropped,
            &placement,
            self.texture_crop_threshold,
        );

        map.get_or_create_texture_state(self.name.as_str())
            .crop_and_put(ui, &request);

        if options.marker_visibility.maps_visible() {
            self.draw_axes(options, Some(&map.pose));
        }
    }

    /// Adds multiple maps to be displayed in the specified draw order.
    pub fn show_maps(
        &self,
        ui: &mut egui::Ui,
        maps: &mut BTreeMap<String, MapState>,
        options: &GridOptions,
        draw_order: &DrawOrder,
    ) {
        for name in draw_order.keys() {
            if let Some(map) = maps.get_mut(name) {
                self.show_map(ui, map, name, options);
            } else {
                error!("Unknown draw order key: {}", name);
            }
        }
    }

    /// Returns the click/drag response of the grid.
    pub fn response(&self) -> &egui::Response {
        &self.response
    }

    /// Returns the mouse pointer position in the grid's metric space, if hovered.
    pub fn hover_pos_metric(&self) -> Option<egui::Pos2> {
        if !self.response.hovered() {
            return None;
        }
        self.response.hover_pos().map(|pos| self.to_metric(&pos))
    }

    /// Drags and zooms the grid according to the input interaction.
    pub fn update_drag_and_zoom(&self, ui: &mut egui::Ui, options: &mut GridOptions) {
        // Scaled because meters are expected for drag().
        options.drag(self.response.drag_delta() / options.scale);
        if self.response.hovered() {
            // Only zoom if the mouse is in the grid region.
            ui.input(|i| {
                let scale_delta = i.smooth_scroll_delta.y * options.scroll_delta_percent;
                if scale_delta != 0. {
                    options.zoom(scale_delta);
                }
            });
        }
    }

    pub fn draw_background(&self, color: egui::Color32) {
        self.painter
            .rect_filled(self.painter.clip_rect(), 0., color);
    }

    fn draw_vertical_lines(
        &self,
        options: &GridOptions,
        line_type: &LineType,
        spacing_points: f32,
        label_text_options: &Option<LabelTextOptions>,
    ) {
        // Calculate how many grid lines we need on each side of the origin.
        let left_bound = self.ui_offset.x;
        let right_bound = self.painter.clip_rect().width() + self.ui_offset.x;
        let left_lines = ((self.origin_in_points.x - left_bound) / spacing_points).ceil() as i32;
        let right_lines = ((right_bound - self.origin_in_points.x) / spacing_points).ceil() as i32;

        // Draw lines using range integers to avoid floating point error accumulation.
        for i in -left_lines..=right_lines {
            let x = self.origin_in_points.x + (i as f32) * spacing_points;
            if x >= left_bound && x <= right_bound {
                self.draw_vertical_line(x, options, line_type, label_text_options);
            }
        }
    }

    fn draw_vertical_line(
        &self,
        x: f32,
        options: &GridOptions,
        line_type: &LineType,
        label_text_options: &Option<LabelTextOptions>,
    ) {
        let stroke = match line_type {
            LineType::Main => &options.line_stroke,
            LineType::Sub => &options.sub_lines_stroke,
        };
        self.painter.line_segment(
            [
                egui::Pos2::new(x, 0.),
                egui::Pos2::new(x, self.painter.clip_rect().height() + self.ui_offset.y),
            ],
            *stroke,
        );
        if !options.tick_labels_visible {
            return;
        }
        if let Some(label_options) = label_text_options {
            let label_pos = egui::Pos2::new(
                x,
                self.painter.clip_rect().height() + self.ui_offset.y - label_options.offset.y,
            );
            // Only draw label if the corresponding line is within the visible bounds.
            if x > self.painter.clip_rect().min.x && x < self.painter.clip_rect().max.x {
                self.painter.text(
                    label_pos,
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
    }

    fn draw_horizontal_lines(
        &self,
        options: &GridOptions,
        line_type: &LineType,
        spacing_points: f32,
        label_text_options: &Option<LabelTextOptions>,
    ) {
        // Calculate how many grid lines we need on each side of the origin.
        let top_bound = self.ui_offset.y;
        let bottom_bound = self.painter.clip_rect().height() + self.ui_offset.y;
        let top_lines = ((self.origin_in_points.y - top_bound) / spacing_points).ceil() as i32;
        let bottom_lines =
            ((bottom_bound - self.origin_in_points.y) / spacing_points).ceil() as i32;

        // Draw lines using range integers to avoid floating point error accumulation.
        for i in -top_lines..=bottom_lines {
            let y = self.origin_in_points.y + (i as f32) * spacing_points;
            if y >= top_bound && y <= bottom_bound {
                self.draw_horizontal_line(y, options, line_type, label_text_options);
            }
        }
    }

    fn draw_horizontal_line(
        &self,
        y: f32,
        options: &GridOptions,
        line_type: &LineType,
        label_text_options: &Option<LabelTextOptions>,
    ) {
        let left = egui::Pos2::new(self.ui_offset.x, y);
        let stroke = match line_type {
            LineType::Main => &options.line_stroke,
            LineType::Sub => &options.sub_lines_stroke,
        };
        self.painter.line_segment(
            [
                left,
                egui::Pos2::new(self.painter.clip_rect().width() + self.ui_offset.x, y),
            ],
            *stroke,
        );
        if !options.tick_labels_visible {
            return;
        }
        if let Some(label_options) = label_text_options {
            // Only draw label if the corresponding line is within the visible bounds.
            if y > self.painter.clip_rect().min.y && y < self.painter.clip_rect().max.y {
                self.painter.text(
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
    }

    /// Draws vertical & horizontal grid lines according to the desired options and line type.
    pub fn draw(&self, options: &GridOptions, line_type: LineType) {
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
                let label_font_size = (spacing_points / 4.).min(15.);
                Some(LabelTextOptions {
                    font_id: egui::FontId::new(label_font_size, egui::FontFamily::Monospace),
                    offset: egui::vec2(0., label_font_size / 2.),
                })
            }
            LineType::Sub => None,
        };

        self.draw_vertical_lines(options, &line_type, spacing_points, &label_text_options);
        self.draw_horizontal_lines(options, &line_type, spacing_points, &label_text_options);
    }

    /// Draws XYZ coordinate axes at the pose.
    pub fn draw_axes(&self, options: &GridOptions, pose: Option<&MapPose>) {
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
            Some(p) => self.to_point(&(p.vec2()).to_pos2()),
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

        self.painter.line_segment([pos, pos + x_vec], x_stroke);
        self.painter.line_segment([pos, pos - y_vec], y_stroke);
        self.painter.circle_filled(
            pos,
            options.marker_width_meters * self.points_per_meter / 2.,
            options.marker_z_color,
        );
    }

    /// Draws the currently active measurement from the `options`, if it exists.
    /// Set `temporary_end` to display an unfinished measurement.
    pub fn draw_measure(&self, options: &GridOptions, temporary_end: Option<egui::Pos2>) {
        if let Some(start_metric) = options.measure_start {
            let start = self.to_point(&start_metric);
            self.painter.circle_filled(
                start,
                options.measure_stroke.width * 2.,
                options.measure_stroke.color,
            );

            let end_metric = options
                .measure_end
                .unwrap_or(temporary_end.unwrap_or(start_metric));
            let end = self.to_point(&end_metric);
            self.painter
                .line_segment([start, end], options.measure_stroke);
            self.painter.circle_filled(
                end,
                options.measure_stroke.width * 2.,
                options.measure_stroke.color,
            );
            self.painter.text(
                end,
                egui::Align2::LEFT_BOTTOM,
                format!("{:.3} m", (end_metric - start_metric).length()),
                egui::FontId::new(15., egui::FontFamily::Monospace),
                options.measure_stroke.color,
            );
        }
    }
}
