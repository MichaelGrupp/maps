use std::option::Option;
use std::vec::Vec;

use eframe::egui;
use image::GenericImageView;
use log::{debug, error};

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
    overlay_texture_handles: Vec<Option<egui::TextureHandle>>,
    desired_size: egui::Vec2,
    hover_region_size_meters: f32,
    hover_region_enabled: bool,
}

impl RosMapsApp {
    pub fn init(metas: Vec<Meta>) -> RosMapsApp {
        RosMapsApp {
            // TODO: probably makes more sense to work with maps here.
            texture_handles: vec![None; metas.len()],
            overlay_texture_handles: vec![None; metas.len()],
            image_pyramids: load_image_pyramids(&metas),
            metas: metas,
            desired_size: egui::Vec2::default(), // Set in show_images.
            hover_region_size_meters: 5.,
            hover_region_enabled: true,
        }
    }

    fn update_desired_size(&mut self, ui: &egui::Ui) {
        let pixels_per_point = ui.ctx().zoom_factor() * ui.ctx().pixels_per_point();
        let desired_size = egui::vec2(
            ui.available_width() * pixels_per_point,
            ui.available_height() * pixels_per_point,
        );
        // TODO: does threshold even make sense?
        let threshold = 0.;
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

    fn update_texture_handles(&mut self, ui: &egui::Ui) {
        self.update_desired_size(ui);
        for (i, texture_handle) in self.texture_handles.iter_mut().enumerate() {
            let texture_name: &str = self.metas[i].image_path.to_str().unwrap();
            texture_handle.get_or_insert_with(|| {
                // Load the texture only if needed.
                debug!("Loading texture for: {}", texture_name);
                let image_pyramid = &self.image_pyramids[i];
                ui.ctx().load_texture(
                    texture_name,
                    to_egui_image(fit_image(
                        image_pyramid
                            .get_level(self.desired_size.max_elem() as u32)
                            .clone(),
                        self.desired_size,
                    )),
                    Default::default(),
                )
            });
        }
    }

    fn show_images(&mut self, ui: &mut egui::Ui) {
        self.update_desired_size(ui);
        self.update_texture_handles(ui);

        for i in 0..self.texture_handles.len() {
            ui.with_layout(egui::Layout::top_down(egui::Align::TOP), |ui| {
                self.show_image(ui, i)
            });
        }
    }

    fn show_image(&mut self, ui: &mut egui::Ui, i: usize) {
        let texture = match &self.texture_handles[i] {
            Some(texture) => texture,
            None => {
                error!("Missing texture handle for image index: {}", i);
                return;
            }
        };
        let response = ui.image(texture);

        let Some(pointer_pos) = response.hover_pos() else {
            // Clear the overlay texture if the mouse is not hovering over the image.
            self.overlay_texture_handles[i] = None;
            return;
        };

        if !self.hover_region_enabled {
            return;
        }

        // Show an overlay with a crop region of the original size image.
        // For this, the pointer position in the rendered texture needs to be converted
        // to corresponding coordinates in the unscaled original image.
        let texture_size = &texture.size_vec2();
        let uv = pointer_pos - response.rect.min;
        let uv = egui::vec2(uv.x / texture_size.x, uv.y / texture_size.y);

        let original_image = &self.image_pyramids[i].original;
        let (original_width, original_height) = original_image.dimensions();
        let original_pos = egui::vec2(uv.x * original_width as f32, uv.y * original_height as f32);

        // Get crop for the overlay.
        let hover_region_size_pixels =
            self.hover_region_size_meters / self.metas[i].resolution as f32;
        let half_region_size = hover_region_size_pixels / 2.;
        let min_x = (original_pos.x - half_region_size).max(0.) as u32;
        let min_y = (original_pos.y - half_region_size).max(0.) as u32;
        let max_x = (original_pos.x + half_region_size).min(original_width as f32) as u32;
        let max_y = (original_pos.y + half_region_size).min(original_height as f32) as u32;
        let cropped_image = original_image.crop_imm(min_x, min_y, max_x - min_x, max_y - min_y);
        let overlay_texture_handle = ui.ctx().load_texture(
            "overlay_".to_owned() + &texture.name(),
            to_egui_image(cropped_image),
            Default::default(),
        );

        // Show the crop area also in the scaled texture coordinates as a small rectangle.
        let stroke = egui::Stroke::new(2., egui::Rgba::from_rgb(0.5, 0.5, 0.));
        let small_rect_ratio = original_width as f32 / texture_size.x as f32;
        let small_rect = egui::Rect::from_min_size(
            pointer_pos - egui::vec2(half_region_size, half_region_size) / small_rect_ratio,
            egui::vec2(hover_region_size_pixels, hover_region_size_pixels) / small_rect_ratio,
        );
        ui.painter()
            .add(egui::Shape::rect_stroke(small_rect, 0., stroke));

        // Display the overlay next to the mouse pointer.
        // Make sure it stays within the window and does not overlap with the small rectangle.
        let pointer_offset = egui::vec2(20., 20.);
        let overlay_pos = (pointer_pos + pointer_offset).min(
            response.rect.max - egui::vec2(hover_region_size_pixels, hover_region_size_pixels),
        );
        let mut overlay_rect = egui::Rect::from_min_size(
            overlay_pos,
            egui::vec2(hover_region_size_pixels, hover_region_size_pixels),
        );
        if overlay_rect.intersects(small_rect) {
            overlay_rect = overlay_rect.translate(egui::vec2(
                -(response.rect.max.x - small_rect.min.x + pointer_offset.x),
                0.,
            ));
        }
        ui.put(overlay_rect, egui::Image::new(&overlay_texture_handle));
        self.overlay_texture_handles[i] = Some(overlay_texture_handle);

        // Draw border around overlay.
        ui.painter().add(egui::Shape::rect_stroke(
            overlay_rect.expand(stroke.width / 2.),
            0.,
            stroke,
        ));
    }
}

impl eframe::App for RosMapsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let space = 10.;
            ui.heading("ROS Maps");
            ui.add_space(space);

            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                ui.label("ROI size (meters):");
                ui.add(egui::Slider::new(
                    &mut self.hover_region_size_meters,
                    2.5..=25.0,
                ));
                ui.checkbox(&mut self.hover_region_enabled, "Show ROI.");
            });

            egui::ScrollArea::both().show(ui, |ui| {
                ui.add_space(space);
                self.show_images(ui);
                // Fill the remaining vertical space, otherwise the scroll bar can jump around.
                ui.add_space(ui.available_height());
            });
        });
    }
}
