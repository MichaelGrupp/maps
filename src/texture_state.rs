use std::option::Option;

use eframe::egui;
use log::debug;

use crate::image::{fit_image, to_egui_image};
use crate::image_pyramid::ImagePyramid;
use crate::texture_request::{RotatedCropRequest, TextureRequest};

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

    pub fn update(&mut self, ui: &egui::Ui, request: &TextureRequest) {
        if self.desired_size != request.desired_rect.size() {
            // Free the old texture if the size changed.
            self.texture_handle = None;
        }
        self.desired_size = request.desired_rect.size();
        self.desired_uv = [egui::Pos2::ZERO, egui::pos2(1., 1.)];
        self.texture_handle.get_or_insert_with(|| {
            // Load the texture only if needed.
            debug!("Fitting and reloading texture for {:?}", request);
            ui.ctx().load_texture(
                request.client.clone(),
                to_egui_image(fit_image(
                    self.image_pyramid.get_level(self.desired_size),
                    self.desired_size,
                )),
                Default::default(),
            )
        });
    }

    pub fn put(&mut self, ui: &mut egui::Ui, request: &TextureRequest) {
        self.update(ui, request);

        match &self.texture_handle {
            Some(texture) => {
                self.image_response = Some(ui.add(egui::Image::new(texture).tint(request.tint)));
            }
            None => {
                panic!("Missing texture handle for {}", request.client)
            }
        }
    }

    pub fn update_crop(&mut self, ui: &mut egui::Ui, request: &RotatedCropRequest) {
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

        debug!("Cropping and reloading texture for {:?}", request);
        let uncropped = self.image_pyramid.get_level(self.desired_size);

        let uv_min = request.uv[0];
        let uv_max = request.uv[1];
        let min_x = (uv_min.x * uncropped.width() as f32).round() as u32;
        let min_y = (uv_min.y * uncropped.height() as f32).round() as u32;
        let max_x = (uv_max.x * uncropped.width() as f32).round() as u32;
        let max_y = (uv_max.y * uncropped.height() as f32).round() as u32;
        let cropped_image = uncropped.crop_imm(min_x, min_y, max_x - min_x, max_y - min_y);
        if cropped_image.width() == 0 || cropped_image.height() == 0 {
            debug!("Crop resulted in empty image.");
            self.texture_handle = None;
            return;
        }

        self.texture_handle = Some(ui.ctx().load_texture(
            request.uncropped.client.clone(),
            to_egui_image(cropped_image),
            Default::default(),
        ));
    }

    pub fn crop_and_put(&mut self, ui: &mut egui::Ui, request: &RotatedCropRequest) {
        self.update_crop(ui, request);

        match &self.texture_handle {
            Some(texture) => {
                // Manually paint and get response.
                // ui.put() clips to the viewport, which is bad for rotated images.
                let image = egui::Image::new(texture)
                    .rotate(request.rotation.angle(), request.rotation_center_in_uv)
                    .maintain_aspect_ratio(false)
                    .fit_to_exact_size(request.visible_rect.size())
                    .tint(request.uncropped.tint);
                image.paint_at(ui, request.visible_rect.translate(request.translation));
                // TODO: this doesn't get the hover response in the rotated texture.
                self.image_response =
                    Some(ui.interact(request.visible_rect, ui.id(), egui::Sense::hover()));
            }
            None => (), // Fine, can be out of view or empty.
        }
    }
}
