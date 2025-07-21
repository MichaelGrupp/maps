use eframe::egui;
use eframe::emath::GuiRounding as _;

use crate::rect_helpers::{debug_paint, quantized_intersection, rotate};
use crate::value_interpretation::ValueInterpretation;

pub const NO_TINT: egui::Color32 = egui::Color32::WHITE;

/// Specifies a request to render an image as a texture inside a desired rectangle
/// with various display options.
#[derive(Debug)]
pub struct TextureRequest {
    /// ID of the requesting client, for texture memory management.
    pub client: String,
    /// The rectangle into which the texture shall be scaled and placed to.
    pub desired_rect: egui::Rect,
    /// Color tint of the texture.
    pub tint: egui::Color32,
    /// Color of the image that shall be displayed as transparent.
    pub color_to_alpha: Option<egui::Color32>,
    /// Optional value-interpretation-based thresholding of the image.
    pub thresholding: Option<ValueInterpretation>,
    /// UI interactions that shall be registered by the image display.
    pub sense: egui::Sense,
    /// Optional overrides for texture rendering options.
    pub texture_options: Option<egui::TextureOptions>,
}

impl TextureRequest {
    pub fn new(client: String, desired_rect: egui::Rect) -> TextureRequest {
        TextureRequest {
            client,
            desired_rect,
            tint: NO_TINT,
            color_to_alpha: None,
            thresholding: None,
            sense: egui::Sense::hover(),
            texture_options: None,
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

    pub fn with_thresholding(
        mut self,
        thresholding: Option<&ValueInterpretation>,
    ) -> TextureRequest {
        self.thresholding = thresholding.copied();
        self
    }

    pub fn with_texture_options(
        mut self,
        texture_options: &egui::TextureOptions,
    ) -> TextureRequest {
        self.texture_options = Some(*texture_options);
        self
    }
}

/// Extended request for rendering scaled textures with arbitrary rotated pose
/// and support for cropping (e.g. to viewport).
#[derive(Debug)]
pub struct RotatedCropRequest {
    /// Base texture request for the bare unrotated & uncropped image rect.
    pub uncropped: TextureRequest,
    /// Bounding box rectangle of the visible area of the final rotated & cropped texture.
    pub visible_rect: egui::Rect,
    /// Image crop specified in UV image coordinates.
    pub uv: [egui::Pos2; 2],
    /// Desired rotation of the texture.
    pub rotation: eframe::emath::Rot2,
    /// Desired translation of the texture.
    pub translation: egui::Vec2,
    /// Rotation center of the image in UV image coordinates.
    pub rotation_center_in_uv: egui::Vec2,
    /// Scale of the texture, i.e. desired screen points per pixel.
    pub points_per_pixel: f32,
}

#[derive(Debug)]
/// Information needed for placement of an image as a scaled texture at a 2D pose.
pub struct ImagePlacement {
    pub rotation: egui::emath::Rot2,
    /// Position of the upper left image corner in points relative to the viewport.
    pub translation: egui::Vec2,
    /// Position of the image's rotation center in points relative to the viewport.
    pub rotation_center: egui::Vec2,
    /// Amount of points occupied by a pixel of the image, for scaling.
    pub points_per_pixel: f32,
    /// Size of the unscaled, uncropped source image in pixels.
    pub original_image_size: egui::Vec2,
}

impl RotatedCropRequest {
    /// Pre-calculate the minimal, unrotated crop that is needed to show the rotated surface in the viewport.
    /// I.e. neither clipping too much nor making the texture unnecessarily large / inefficient.
    /// Enable trace log level to see what is going on (I spent too much time figuring this out).
    fn min_crop(
        ui: &egui::Ui,
        viewport_clip_rect: &egui::Rect,
        image_rect: &egui::Rect,
        rotation: &eframe::emath::Rot2,
        translation: &egui::Vec2,
        rotation_center_in_points: &egui::Vec2,
        points_per_pixel: f32,
    ) -> egui::Rect {
        let origin_in_points = (image_rect.min - *rotation_center_in_points).to_vec2();

        let rotated = rotate(image_rect, *rotation, origin_in_points);
        let transformed = rotated.translate(*translation);
        debug_paint(ui, transformed, egui::Color32::RED, "transformed");

        let transformed_visible = transformed.intersect(*viewport_clip_rect);
        debug_paint(
            ui,
            transformed_visible,
            egui::Color32::GOLD,
            "transformed_visible",
        );

        let min_crop = rotate(
            &transformed_visible.translate(-*translation),
            rotation.inverse(),
            origin_in_points,
        );
        debug_paint(ui, min_crop, egui::Color32::BLUE, "min_crop");

        // The minimal rectangle is the instersection of crop rectangle and image rectangle.
        // The image cropping happens in pixel space, so we have to also quantize the rectangle
        // to the next best multiple of the scaled pixel size.
        // Otherwise the texture size/placement is not exact, especially at high zoom levels.
        let visible_rect = quantized_intersection(image_rect, &min_crop, points_per_pixel);
        debug_paint(
            ui,
            visible_rect,
            egui::Color32::GREEN,
            "visible_rect_quantized",
        );
        visible_rect
    }

    /// Creates a request for displaying an image with the desired `placement`
    /// in the visible viewport of the `ui`.
    /// `crop_threshold` controls the maximum size of a texture before it gets
    /// cropped to the viewport. Use this to support displaying large images
    /// at high zoom levels as cropped textures to avoid texture buffer size limits.
    pub fn from_visible(
        ui: &egui::Ui,
        viewport_clip_rect: &egui::Rect,
        uncropped: TextureRequest,
        placement: &ImagePlacement,
        crop_threshold: u32,
    ) -> RotatedCropRequest {
        let image_rect = uncropped.desired_rect;
        let visible_rect = if uncropped.desired_rect.size().max_elem() as u32 <= crop_threshold
            || placement.original_image_size.max_elem() as u32 <= crop_threshold
        {
            // Desired texture is small enough to not need cropping.
            image_rect
        } else {
            // Desired texture is large, crop to the viewport.
            Self::min_crop(
                ui,
                viewport_clip_rect,
                &image_rect,
                &placement.rotation,
                &placement.translation,
                &placement.rotation_center,
                placement.points_per_pixel,
            )
        };

        // Round visible_rect matching egui 0.32's "pixel-perfect" paint_at behavior.
        // See also: https://github.com/emilk/egui/pull/7078
        let pixels_per_point = ui.pixels_per_point();
        let visible_rect = visible_rect.round_to_pixels(pixels_per_point);

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
            rotation: placement.rotation,
            translation: placement.translation,
            rotation_center_in_uv: egui::Vec2::new(
                -(placement.rotation_center.x + (visible_rect.min.x - image_rect.min.x))
                    / visible_rect.width(),
                -(placement.rotation_center.y + (visible_rect.min.y - image_rect.min.y))
                    / visible_rect.height(),
            ),
            points_per_pixel: placement.points_per_pixel,
        }
    }
}
