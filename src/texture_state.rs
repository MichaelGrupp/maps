use std::option::Option;

use eframe::egui;
use log::debug;

use crate::image::{fit_image, to_egui_image};
use crate::image_pyramid::ImagePyramid;

#[derive(Default)]
pub struct TextureState {
    pub image_pyramid: ImagePyramid,
    pub image_response: Option<egui::Response>,
    pub texture_handle: Option<egui::TextureHandle>,
    pub desired_size: egui::Vec2,
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

    pub fn update(&mut self, ui: &egui::Ui, name: &str) {
        self.update_desired_size(ui);
        self.texture_handle.get_or_insert_with(|| {
            // Load the texture only if needed.
            debug!("Fitting and reloading texture for {}", name);
            ui.ctx().load_texture(
                name,
                to_egui_image(fit_image(
                    self.image_pyramid
                        .get_level(self.desired_size.max_elem() as u32)
                        .clone(),
                    self.desired_size,
                )),
                Default::default(),
            )
        });
    }
}
