use std::option::Option;

use eframe::egui;
use log::debug;

use crate::image::{fit_image, to_egui_image};
use crate::image_pyramid::ImagePyramid;
use crate::texture_request::CropRequest;

#[derive(Default)]
pub struct TextureState {
    pub image_pyramid: ImagePyramid,
    pub image_response: Option<egui::Response>,
    pub texture_handle: Option<egui::TextureHandle>,
    pub desired_size: egui::Vec2,
    pub desired_uv: [egui::Pos2; 2],
}

impl TextureState {
    pub fn new(image_pyramid: ImagePyramid) -> TextureState {
        TextureState {
            image_pyramid,
            ..Default::default()
        }
    }

    fn update_desired_size(&mut self, ui: &egui::Ui) {
        let old_size = self.desired_size;
        let pixels_per_point = ui.ctx().zoom_factor() * ui.ctx().pixels_per_point();
        let desired_size = egui::vec2(
            ui.available_width() * pixels_per_point,
            ui.available_height() * pixels_per_point,
        );
        if desired_size != old_size {
            // Note that in egui dropping the last handle of a texture will free it.
            debug!(
                "Desired size changed to {:?}, clearing texture.",
                desired_size
            );
            self.texture_handle = None;
        }
        self.desired_size = desired_size;
    }

    pub fn update_to_available_space(&mut self, ui: &egui::Ui, name: &str) {
        self.update_desired_size(ui);
        self.update(ui, name);
    }

    pub fn update(&mut self, ui: &egui::Ui, name: &str) {
        self.texture_handle.get_or_insert_with(|| {
            // Load the texture only if needed.
            debug!("Fitting and reloading texture for {}", name);
            ui.ctx().load_texture(
                name,
                to_egui_image(fit_image(
                    self.image_pyramid.get_level(self.desired_size),
                    self.desired_size,
                )),
                Default::default(),
            )
        });
    }

    pub fn update_crop(&mut self, ui: &mut egui::Ui, request: &CropRequest) {
        let desired_size = request.uncropped.desired_rect.size();
        if self.desired_size == desired_size && self.desired_uv == request.uv {
            return;
        }
        self.desired_size = desired_size;
        self.desired_uv = request.uv;

        if request.visible_rect.is_negative() || request.uv[0] == request.uv[1] {
            self.texture_handle = None;
            return;
        }

        let uncropped = self.image_pyramid.get_level(self.desired_size);

        let uv_min = request.uv[0];
        let uv_max = request.uv[1];
        let min_x = (uv_min.x * uncropped.width() as f32).round() as u32;
        let min_y = (uv_min.y * uncropped.height() as f32).round() as u32;
        let max_x = (uv_max.x * uncropped.width() as f32).round() as u32;
        let max_y = (uv_max.y * uncropped.height() as f32).round() as u32;
        let cropped_image = uncropped.crop_imm(min_x, min_y, max_x - min_x, max_y - min_y);

        self.texture_handle = Some(ui.ctx().load_texture(
            request.uncropped.client.clone(),
            to_egui_image(cropped_image),
            Default::default(),
        ));
    }

    pub fn crop_and_put(&mut self, ui: &mut egui::Ui, request: &CropRequest) {
        self.update_crop(ui, request);

        match &self.texture_handle {
            Some(texture) => {
                self.image_response = Some(
                    ui.put(
                        request.visible_rect,
                        egui::Image::new(texture)
                            .maintain_aspect_ratio(false)
                            .fit_to_exact_size(request.visible_rect.size()),
                    ),
                );
            }
            None => (), // Fine, can be out of view or empty.
        }
    }
}
