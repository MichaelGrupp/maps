use std::option::Option;
use std::vec::Vec;

use eframe::egui;
use image::GenericImageView;
use log::debug;

use crate::image::{fit_image, load_image, to_egui_image};
use crate::image_pyramid::ImagePyramid;
use crate::meta::Meta;

fn load_image_pyramids(metas: &Vec<Meta>) -> Vec<ImagePyramid> {
    metas
        .iter()
        .map(|meta| ImagePyramid::new(load_image(&meta.image_path)))
        .collect()
}

#[derive(Default)]
pub struct RosMapsApp {
    metas: Vec<Meta>,
    image_pyramids: Vec<ImagePyramid>,
    texture_handles: Vec<Option<egui::TextureHandle>>,
    overlay_texture_handle: Option<egui::TextureHandle>,
    desired_size: egui::Vec2,
    hover_region_size: f32,
}

impl RosMapsApp {
    pub fn init(metas: Vec<Meta>) -> RosMapsApp {
        RosMapsApp {
            // TODO: probably makes more sense to work with maps here.
            texture_handles: vec![None; metas.len()],
            overlay_texture_handle: None,
            image_pyramids: load_image_pyramids(&metas),
            metas: metas,
            desired_size: egui::Vec2::default(), // Set in show_images.
            hover_region_size: 250.,
        }
    }

    fn update_desired_size(&mut self, ui: &egui::Ui) {
        let pixels_per_point = ui.ctx().zoom_factor() * ui.ctx().pixels_per_point();
        // TODO: this is probably not the exact size we want.
        let viewport_info = ui.ctx().screen_rect();
        let desired_size = egui::vec2(
            viewport_info.width() * pixels_per_point,
            viewport_info.height() * pixels_per_point,
        );
        let threshold = 5.;
        if (desired_size.x - self.desired_size.x).abs() > threshold
            || (desired_size.y - self.desired_size.y).abs() > threshold
        {
            // Clear the texture handles if the size changes "significantly".
            // Note that in egui dropping the last handle of a texture will free it.
            debug!(
                "Desired size changed to {:?}, clearing textures.",
                desired_size
            );
            self.texture_handles = vec![None; self.metas.len()];
        }
        self.desired_size = desired_size;
    }

    fn show_images(&mut self, ui: &mut egui::Ui) {
        self.update_desired_size(ui);
        for (i, texture_handle) in self.texture_handles.iter_mut().enumerate() {
            let texture: &egui::TextureHandle = texture_handle.get_or_insert_with(|| {
                // Load the texture only if needed.
                let name: &str = self.metas[i].image_path.to_str().unwrap();
                debug!("Loading texture for: {}", name);
                let image_pyramid = &self.image_pyramids[i];
                ui.ctx().load_texture(
                    name,
                    to_egui_image(fit_image(
                        image_pyramid
                            .get_level(self.desired_size.max_elem() as u32)
                            .clone(),
                        self.desired_size,
                    )),
                    Default::default(),
                )
            });
            let response = ui.image(texture);

            ui.add(egui::Slider::new(
                &mut self.hover_region_size,
                100.0..=1000.0,
            ));
            ui.separator();

            if response.hovered() {
                if let Some(pointer_pos) = response.hover_pos() {
                    ui.label(format!("Pointer position (window): {:?}", pointer_pos));
                    let texture_size = texture.size_vec2();
                    let uv = pointer_pos - response.rect.min;
                    let uv = egui::vec2(uv.x / texture_size.x, uv.y / texture_size.y);
                    let pixel_pos = egui::vec2(uv.x * texture_size.x, uv.y * texture_size.y);
                    ui.label(format!("Pixel position: {:?}", pixel_pos));

                    // Calculate the region of the original image to display.
                    let half_region_size = self.hover_region_size / 2.;
                    let original_image = &self.image_pyramids[i].original;
                    let (original_width, original_height) = original_image.dimensions();
                    let original_pos =
                        egui::vec2(uv.x * original_width as f32, uv.y * original_height as f32);
                    ui.label(format!("Original pixel position: {:?}", original_pos));
                    let min_x = (original_pos.x - half_region_size).max(0.) as u32;
                    let min_y = (original_pos.y - half_region_size).max(0.) as u32;
                    let max_x =
                        (original_pos.x + half_region_size).min(original_width as f32) as u32;
                    let max_y =
                        (original_pos.y + half_region_size).min(original_height as f32) as u32;

                    // Get crop for the overlay.
                    let cropped_image =
                        original_image.crop_imm(min_x, min_y, max_x - min_x, max_y - min_y);
                    let overlay_texture_handle = ui.ctx().load_texture(
                        "overlay",
                        to_egui_image(cropped_image),
                        Default::default(),
                    );

                    // Display the overlay centered at the mouse pointer.
                    let overlay_pos = pointer_pos + egui::vec2(10., 10.);
                    ui.put(
                        egui::Rect::from_min_size(
                            overlay_pos,
                            egui::vec2(self.hover_region_size, self.hover_region_size),
                        ),
                        egui::Image::new(&overlay_texture_handle),
                    );
                    self.overlay_texture_handle = Some(overlay_texture_handle);
                }
            } else {
                // Clear the overlay texture if the mouse is not hovering over the image.
                self.overlay_texture_handle = None;
            }
        }
    }
}

impl eframe::App for RosMapsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("ROS Maps");
            egui::ScrollArea::both().show(ui, |ui| {
                self.show_images(ui);
            });
            ctx.pointer_hover_pos().map(|pos| {
                ui.label(format!("Mouse position: {:?}", pos));
            });
        });
    }
}
