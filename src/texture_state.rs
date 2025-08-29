use std::sync::Arc;

use eframe::egui;
use log::trace;

use crate::image::{color_to_alpha, fit_image, to_egui_image};
use crate::image_pyramid::ImagePyramid;
use crate::texture_cache::TextureCache;
use crate::texture_request::{RotatedCropRequest, TextureRequest};
use crate::value_interpretation::ValueInterpretation;

/// Manages the state of a texture across its lifetime.
/// Has to be updated every frame using texture requests.
#[derive(Default)]
pub struct TextureState {
    /// Image pyramid with source images for different zoom levels.
    // Image pyramid is shared to avoid duplicating it.
    // Use init() to set it.
    pub image_pyramid: Arc<ImagePyramid>,
    pub image_response: Option<egui::Response>,
    /// Cache of textures for different sizes and appearance settings.
    texture_cache: TextureCache,
    /// Currently active texture (reference to one in the cache).
    pub texture_handle: Option<egui::TextureHandle>,
    pub desired_size: egui::Vec2,
    pub desired_uv: [egui::Pos2; 2],
    pub desired_color_to_alpha: Option<egui::Color32>,
    pub desired_thresholding: Option<ValueInterpretation>,
    pub used_level: u32,
    pub texture_options: egui::TextureOptions,
}

impl TextureState {
    pub fn new(image_pyramid: Arc<ImagePyramid>) -> TextureState {
        TextureState {
            image_pyramid,
            texture_cache: TextureCache::new(),
            ..Default::default()
        }
    }

    /// Returns true if the request changes the texture and requires re-rendering.
    fn changed(&self, request: &TextureRequest) -> bool {
        self.desired_size != request.desired_rect.size() || self.changed_appearance(request)
    }

    /// Returns true if the appearance of the texture changed (not checking size).
    fn changed_appearance(&self, request: &TextureRequest) -> bool {
        self.desired_color_to_alpha != request.color_to_alpha
            || self.desired_thresholding != request.thresholding
            || self.texture_options != request.texture_options.unwrap_or_default()
    }

    /// Updates the texture state for a new incoming request, if needed.
    /// Chooses the appropriate level from the image pyramid.
    pub fn update(&mut self, ui: &egui::Ui, request: &TextureRequest) {
        if self.changed(request) {
            // Free the old texture if the size changed.
            self.texture_handle = None;
        }
        self.desired_size = request.desired_rect.size();
        self.desired_uv = [egui::Pos2::ZERO, egui::pos2(1., 1.)];
        self.desired_color_to_alpha = request.color_to_alpha;
        self.desired_thresholding = request.thresholding;
        self.texture_options = request.texture_options.unwrap_or_default();
        self.texture_handle.get_or_insert_with(|| {
            // Load the texture only if needed.
            trace!("Fitting and reloading texture for {:?}", request);
            let mut image = fit_image(
                self.image_pyramid.get_level(self.desired_size),
                self.desired_size,
            );
            color_to_alpha(&mut image, request.color_to_alpha);
            if let Some(thresholding) = &request.thresholding {
                thresholding.apply(&mut image, self.image_pyramid.original_has_alpha);
            }
            ui.ctx().load_texture(
                request.client.clone(),
                to_egui_image(image),
                self.texture_options,
            )
        });
    }

    /// Updates the state and puts the texture into the UI according to the request.
    pub fn put(&mut self, ui: &mut egui::Ui, request: &TextureRequest) {
        self.update(ui, request);

        match &self.texture_handle {
            Some(texture) => {
                self.image_response = Some(
                    ui.add(egui::Image::new(texture).tint(request.tint))
                        .interact(request.sense),
                );
            }
            None => {
                panic!("Missing texture handle for {}", request.client)
            }
        }
    }

    /// Returns true if the request changes the image cropping.
    fn changed_crop(&self, request: &RotatedCropRequest) -> bool {
        self.desired_uv != request.uv
    }

    /// Tries to find and use a cached texture for full (non-cropped) textures.
    /// Returns true if a cached texture was found and applied, false otherwise.
    fn try_use_cached_texture(
        &mut self,
        request: &RotatedCropRequest,
        desired_size: egui::Vec2,
    ) -> bool {
        // Only cache full textures (not crops).
        if !request.is_full_texture() {
            return false;
        }

        let uncropped = self.image_pyramid.get_level(desired_size);
        let level = uncropped.width().max(uncropped.height());

        // Check if we have a cached texture that matches appearance.
        if let Some(texture_handle) =
            self.texture_cache
                .query(&request.uncropped.client, level, &request.uncropped)
        {
            // Reuse cached texture and update state.
            self.texture_handle = Some(texture_handle);
            self.used_level = level;
            self.desired_size = desired_size;
            self.desired_uv = request.uv;
            self.desired_color_to_alpha = request.uncropped.color_to_alpha;
            self.desired_thresholding = request.uncropped.thresholding;
            self.texture_options = request.uncropped.texture_options.unwrap_or_default();
            return true;
        }

        false
    }

    /// Updates the texture state for a new incoming crop/rotate request, if needed.
    /// Chooses the appropriate level from the image pyramid and crops if required.
    /// Full (non-cropped) textures are cached to avoid reloading when zooming.
    ///
    /// Process:
    /// 1. Try to reuse cached texture for full textures.
    /// 2. Check if any changes require creating a new texture.
    /// 3. Create and optionally cache the new texture.
    pub fn update_crop(&mut self, ui: &mut egui::Ui, request: &RotatedCropRequest) {
        let desired_size = request.uncropped.desired_rect.size();

        // Try to use cached texture for full textures.
        if self.try_use_cached_texture(request, desired_size) {
            return;
        }

        // Check if we need to create a new texture.
        let changed_uncropped = self.changed(&request.uncropped);
        let changed_crop = self.changed_crop(request);
        let changed_appearance = self.changed_appearance(&request.uncropped);

        if !(changed_uncropped || changed_crop || changed_appearance) {
            return;
        }

        self.desired_size = desired_size;
        self.desired_uv = request.uv;
        self.desired_color_to_alpha = request.uncropped.color_to_alpha;
        self.desired_thresholding = request.uncropped.thresholding;
        self.texture_options = request.uncropped.texture_options.unwrap_or_default();

        if request.visible_rect.is_negative() || request.uv[0] == request.uv[1] {
            self.texture_handle = None;
            return;
        }

        let uncropped = self.image_pyramid.get_level(self.desired_size);
        let level = uncropped.width().max(uncropped.height());
        self.used_level = level;

        trace!("Cropping and reloading texture for {:?}", request);
        let uv_min = request.uv[0];
        let uv_max = request.uv[1];
        let min_x = (uv_min.x * uncropped.width() as f32).round() as u32;
        let min_y = (uv_min.y * uncropped.height() as f32).round() as u32;
        let max_x = (uv_max.x * uncropped.width() as f32).round() as u32;
        let max_y = (uv_max.y * uncropped.height() as f32).round() as u32;
        let mut cropped_image = uncropped.crop_imm(min_x, min_y, max_x - min_x, max_y - min_y);
        if cropped_image.width() == 0 || cropped_image.height() == 0 {
            trace!("Crop resulted in empty image.");
            self.texture_handle = None;
            return;
        }
        color_to_alpha(&mut cropped_image, request.uncropped.color_to_alpha);
        if let Some(thresholding) = &request.uncropped.thresholding {
            thresholding.apply(&mut cropped_image, self.image_pyramid.original_has_alpha);
        }

        let texture_handle = ui.ctx().load_texture(
            format!("{}_{}", request.uncropped.client, level),
            to_egui_image(cropped_image),
            self.texture_options,
        );

        // Cache full textures for zoom performance.
        if request.is_full_texture() {
            self.texture_cache.store(
                &request.uncropped.client,
                level,
                texture_handle.clone(),
                &request.uncropped,
            );
        }

        self.texture_handle = Some(texture_handle);
    }

    /// Updates the state and puts the texture into the UI according to the request.
    pub fn crop_and_put(&mut self, ui: &mut egui::Ui, request: &RotatedCropRequest) {
        self.update_crop(ui, request);

        if let Some(texture) = &self.texture_handle {
            // Manually paint and get response.
            // ui.put() clips to the viewport, which is bad for rotated images.
            let image = egui::Image::new(texture)
                .rotate(request.rotation.angle(), request.rotation_center_in_uv)
                .maintain_aspect_ratio(false)
                .fit_to_exact_size(request.visible_rect.size())
                .tint(request.uncropped.tint);
            image.paint_at(ui, request.visible_rect.translate(request.translation));
            // We can't get a proper image response from a rotated/translated manual paint,
            // and also don't need one (grid interaction is handled elsewhere).
            self.image_response = None;
        }
    }
}
