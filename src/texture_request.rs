use eframe::egui;

use crate::rect_helpers::{debug_paint, rotate};

pub const NO_TINT: egui::Color32 = egui::Color32::WHITE;

#[derive(Debug)]
pub struct TextureRequest {
    pub client: String,
    pub desired_rect: egui::Rect,
    pub tint: egui::Color32,
    pub color_to_alpha: Option<egui::Color32>,
    pub sense: egui::Sense,
}

impl TextureRequest {
    pub fn new(client: String, desired_rect: egui::Rect) -> TextureRequest {
        TextureRequest {
            client,
            desired_rect,
            tint: NO_TINT,
            color_to_alpha: None,
            sense: egui::Sense::hover(),
        }
    }

    pub fn with_sense(mut self, sense: egui::Sense) -> TextureRequest {
        self.sense = sense;
        self
    }

    pub fn with_tint(mut self, tint: Option<egui::Color32>) -> TextureRequest {
        match tint {
            Some(tint) => {
                self.tint = tint;
            }
            None => {
                self.tint = NO_TINT;
            }
        }
        self
    }

    pub fn with_color_to_alpha(mut self, color_to_alpha: Option<egui::Color32>) -> TextureRequest {
        self.color_to_alpha = color_to_alpha;
        self
    }
}

#[derive(Debug)]
pub struct RotatedCropRequest {
    pub uncropped: TextureRequest,
    pub visible_rect: egui::Rect,
    pub uv: [egui::Pos2; 2],
    pub rotation: eframe::emath::Rot2,
    pub translation: egui::Vec2,
    pub rotation_center_in_uv: egui::Vec2,
}

impl RotatedCropRequest {
    pub fn from_visible(
        ui: &egui::Ui,
        uncropped: TextureRequest,
        rotation: egui::emath::Rot2,
        translation: egui::Vec2,
        rotation_center_in_points: egui::Vec2,
    ) -> RotatedCropRequest {
        let viewport_rect = ui.clip_rect();
        let image_rect = uncropped.desired_rect;
        let origin_in_points = (image_rect.min - rotation_center_in_points).to_vec2();

        // Pre-calculate the minimal, unrotated crop that is needed to show the rotated surface in the viewport.
        // I.e. neither clipping too much nor making the texture unnecessarily large / inefficient.
        // Enable debug log level to see what is going on (I spent too much time figuring this out).
        let rotated = rotate(&image_rect, rotation, origin_in_points);
        let transformed = rotated.translate(translation);
        debug_paint(ui, transformed, egui::Color32::RED);

        let transformed_visible = transformed.intersect(viewport_rect);
        debug_paint(ui, transformed_visible, egui::Color32::GOLD);

        let min_crop = rotate(
            &transformed_visible.translate(-translation),
            rotation.inverse(),
            origin_in_points,
        );
        debug_paint(ui, min_crop, egui::Color32::BLUE);

        let visible_rect = min_crop.intersect(image_rect);
        debug_paint(ui, visible_rect, egui::Color32::GREEN);

        RotatedCropRequest {
            uncropped,
            visible_rect,
            uv: [
                egui::Pos2::new(
                    (visible_rect.min.x - image_rect.min.x) / image_rect.width(),
                    (visible_rect.min.y - image_rect.min.y) / image_rect.height(),
                ),
                egui::Pos2::new(
                    (visible_rect.max.x - image_rect.min.x) / image_rect.width(),
                    (visible_rect.max.y - image_rect.min.y) / image_rect.height(),
                ),
            ],
            rotation,
            translation,
            rotation_center_in_uv: egui::Vec2::new(
                -(rotation_center_in_points.x + (visible_rect.min.x - image_rect.min.x))
                    / visible_rect.width(),
                -(rotation_center_in_points.y + (visible_rect.min.y - image_rect.min.y))
                    / visible_rect.height(),
            ),
        }
    }
}
