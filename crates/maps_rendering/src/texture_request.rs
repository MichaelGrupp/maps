use eframe::egui;
use eframe::emath::GuiRounding as _;

use crate::rect_helpers::{debug_paint, quantized_intersection, rotate_aabb};
use maps_io_ros::ValueInterpretation;

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

    pub fn with_texture_options(mut self, texture_options: egui::TextureOptions) -> TextureRequest {
        self.texture_options = Some(texture_options);
        self
    }
}

/// Extended request for rendering scaled textures with arbitrary rotated pose
/// and support for cropping (e.g. to viewport).
#[derive(Debug)]
pub struct TransformedTextureRequest {
    /// Base texture request for the bare unrotated & untransformed image rect.
    pub base_request: TextureRequest,
    /// Rectangle in the scaled image coordinate space defining the
    /// potentially cropped region to extract before applying transformations.
    pub crop_rect: egui::Rect,
    /// Image crop specified in UV image coordinates.
    pub crop_uv: [egui::Pos2; 2],
    /// Desired rotation of the texture.
    pub rotation: eframe::emath::Rot2,
    /// Desired translation of the texture.
    pub translation: egui::Vec2,
    /// Rotation center of the image in UV image coordinates.
    pub rotation_center_in_uv: egui::Vec2,
    /// Scale of the texture, i.e. desired screen points per texel.
    pub points_per_texel: f32,
}

/// Information needed for placement of an image as a scaled texture at a 2D pose.
#[derive(Debug)]
pub struct ImagePlacement {
    pub rotation: egui::emath::Rot2,
    /// Position of the upper left image corner in points relative to the viewport.
    pub translation: egui::Vec2,
    /// Position of the image's rotation center in points relative to the viewport.
    pub rotation_center: egui::Vec2,
    /// Amount of points occupied by a texel of the image, for scaling.
    pub points_per_texel: f32,
    /// Size of the unscaled, uncropped source image in pixels.
    pub original_image_size: egui::Vec2,
}

impl TransformedTextureRequest {
    /// Returns true if this request represents a full texture (not a crop).
    pub fn is_full_texture(&self) -> bool {
        self.crop_uv[0] == egui::Pos2::ZERO && self.crop_uv[1] == egui::pos2(1.0, 1.0)
    }

    /// Pre-calculate the minimal, unrotated crop that is needed to show the rotated surface in the viewport.
    /// I.e. neither clipping too much nor making the texture unnecessarily large / inefficient.
    /// Enable trace log level to see what is going on (I spent too much time figuring this out).
    fn min_crop(
        paint_context: &egui::Painter,
        scaled_rect: &egui::Rect,
        rotation: eframe::emath::Rot2,
        translation: egui::Vec2,
        rotation_center_in_points: egui::Vec2,
        points_per_texel: f32,
    ) -> egui::Rect {
        let origin_in_points = (scaled_rect.min - rotation_center_in_points).to_vec2();

        let transformed_aabb =
            rotate_aabb(scaled_rect, rotation, origin_in_points).translate(translation);
        debug_paint(
            paint_context,
            transformed_aabb,
            egui::Color32::RED,
            "transformed_aabb",
        );

        let transformed_aabb_visible = transformed_aabb.intersect(paint_context.clip_rect());
        debug_paint(
            paint_context,
            transformed_aabb_visible,
            egui::Color32::GOLD,
            "transformed_aabb_visible",
        );

        let min_crop = rotate_aabb(
            &transformed_aabb_visible.translate(-translation),
            rotation.inverse(),
            origin_in_points,
        );
        let ui_offset = paint_context.clip_rect().min.to_vec2();
        debug_paint(
            paint_context,
            min_crop.translate(ui_offset),
            egui::Color32::BLUE,
            "min_crop",
        );

        // The minimal rectangle is the instersection of crop rectangle and image rectangle.
        // The image cropping happens in pixel space, so we have to also quantize the rectangle
        // to the next best multiple of the scaled pixel size.
        // Otherwise the texture size/placement is not exact, especially at high zoom levels.
        let crop_rect = quantized_intersection(scaled_rect, &min_crop, points_per_texel);
        // Round crop_rect matching egui 0.32's "pixel-perfect" paint_at behavior.
        // See also: https://github.com/emilk/egui/pull/7078
        let crop_rect = crop_rect.round_to_pixels(paint_context.ctx().pixels_per_point());
        debug_paint(
            paint_context,
            crop_rect.translate(ui_offset),
            egui::Color32::GREEN,
            "crop_rect_quantized",
        );
        crop_rect
    }

    /// Creates a request for displaying an image with the desired `placement`
    /// in the visible viewport of the `ui`.
    /// `crop_threshold` controls the maximum size of a texture before it gets
    /// cropped to the viewport. Use this to support displaying large images
    /// at high zoom levels as cropped textures to avoid texture buffer size limits.
    pub fn from_visible(
        paint_context: &egui::Painter,
        base_request: TextureRequest,
        placement: &ImagePlacement,
        crop_threshold: u32,
    ) -> TransformedTextureRequest {
        let scaled_rect = base_request.desired_rect;
        let crop_rect = if scaled_rect.size().max_elem() as u32 <= crop_threshold
            || placement.original_image_size.max_elem() as u32 <= crop_threshold
        {
            // Desired texture is small enough to not need cropping.
            scaled_rect
        } else {
            // Desired texture is large, crop to the viewport.
            Self::min_crop(
                paint_context,
                &scaled_rect,
                placement.rotation,
                placement.translation,
                placement.rotation_center,
                placement.points_per_texel,
            )
        };

        TransformedTextureRequest {
            base_request,
            crop_rect,
            crop_uv: [
                egui::Pos2::new(
                    (crop_rect.min.x - scaled_rect.min.x) / scaled_rect.width(),
                    (crop_rect.min.y - scaled_rect.min.y) / scaled_rect.height(),
                ),
                egui::Pos2::new(
                    (crop_rect.max.x - scaled_rect.min.x) / scaled_rect.width(),
                    (crop_rect.max.y - scaled_rect.min.y) / scaled_rect.height(),
                ),
            ],
            rotation: placement.rotation,
            translation: placement.translation,
            rotation_center_in_uv: egui::Vec2::new(
                -(placement.rotation_center.x + (crop_rect.min.x - scaled_rect.min.x))
                    / crop_rect.width(),
                -(placement.rotation_center.y + (crop_rect.min.y - scaled_rect.min.y))
                    / crop_rect.height(),
            ),
            points_per_texel: placement.points_per_texel,
        }
    }
}
