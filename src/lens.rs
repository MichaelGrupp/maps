use std::default;

use eframe::egui;
use image::GenericImageView;
use log::debug;

use crate::image::to_egui_image;
use crate::map_state::MapState;

pub struct Lens {
    pub enabled: bool,
    pub size_meters: f32,
    pub size_meters_min: f32,
    pub size_meters_max: f32,
    pub scroll_speed_factor: f32,
}

impl default::Default for Lens {
    fn default() -> Lens {
        Lens {
            enabled: true,
            size_meters: 5.,
            size_meters_min: 2.5,
            size_meters_max: 25.,
            scroll_speed_factor: 0.2,
        }
    }
}

impl Lens {
    pub fn show_on_hover(&mut self, ui: &mut egui::Ui, map: &mut MapState, name: &str) {
        if !self.enabled {
            return;
        }

        let response = match &map.texture_state.image_response {
            Some(response) => response,
            None => {
                // Can be missing e.g. if a tab is not visible yet.
                return;
            }
        };

        let Some(pointer_pos) = response.hover_pos() else {
            // Clear the overlay texture if the mouse is not hovering over the image.
            map.overlay_texture = None;
            return;
        };

        ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);

        // Change the hover region size when scrolling.
        self.size_meters = (self.size_meters
            + ui.input(|i| i.smooth_scroll_delta).y * self.scroll_speed_factor)
            .clamp(self.size_meters_min, self.size_meters_max);

        // Show an overlay with a crop region of the original size image.
        // For this, the pointer position in the rendered texture needs to be converted
        // to corresponding coordinates in the unscaled original image.
        let texture = match &map.texture_state.texture_handle {
            Some(texture) => texture,
            None => {
                panic!("Missing texture handle for image {}", name);
            }
        };
        let texture_size = &texture.size_vec2();
        let uv = pointer_pos - response.rect.min;
        let uv = egui::vec2(uv.x / texture_size.x, uv.y / texture_size.y);

        let original_image = &map.image_pyramid.original;
        let (original_width, original_height) = original_image.dimensions();
        let original_pos = egui::vec2(uv.x * original_width as f32, uv.y * original_height as f32);

        // Get crop for the overlay.
        let hover_region_size_pixels = self.size_meters / map.meta.resolution as f32;
        let half_region_size = hover_region_size_pixels / 2.;
        let min_x = (original_pos.x - half_region_size).max(0.) as u32;
        let min_y = (original_pos.y - half_region_size).max(0.) as u32;
        let max_x = (original_pos.x + half_region_size).min(original_width as f32) as u32;
        let max_y = (original_pos.y + half_region_size).min(original_height as f32) as u32;
        if min_x >= max_x || min_y >= max_y {
            debug!("Ignoring hover because region would be empty.");
            return;
        }
        let cropped_image = original_image.crop_imm(min_x, min_y, max_x - min_x, max_y - min_y);
        let overlay_texture_handle = ui.ctx().load_texture(
            "overlay_".to_owned() + name,
            to_egui_image(cropped_image),
            Default::default(),
        );

        // Show the crop area also in the scaled texture coordinates as a small rectangle.
        let stroke = egui::Stroke::new(2., egui::Rgba::from_rgb(0., 0., 0.));
        let small_rect_ratio = original_width as f32 / texture_size.x as f32;
        let small_rect = egui::Rect::from_min_size(
            pointer_pos - egui::vec2(half_region_size, half_region_size) / small_rect_ratio,
            egui::vec2(hover_region_size_pixels, hover_region_size_pixels) / small_rect_ratio,
        );
        ui.painter()
            .add(egui::Shape::rect_stroke(small_rect, 0., stroke));

        // Display the overlay next to the mouse pointer.
        // Make sure it stays within the window and does not overlap with the small rectangle.
        let pointer_offset = egui::vec2(small_rect.width(), small_rect.width());
        let overlay_pos = (pointer_pos + pointer_offset).min(
            response.rect.max - egui::vec2(hover_region_size_pixels, hover_region_size_pixels),
        );
        let mut overlay_rect = egui::Rect::from_min_size(
            overlay_pos,
            egui::vec2(hover_region_size_pixels, hover_region_size_pixels),
        );
        if overlay_rect.intersects(small_rect) {
            let distance_to_right = response.rect.max.x - small_rect.max.x;
            overlay_rect = overlay_rect.translate(egui::vec2(
                -(distance_to_right + small_rect.width() + pointer_offset.x),
                0.,
            ));
        }

        egui::Window::new("overlay_window")
            .title_bar(false)
            .auto_sized()
            .current_pos(overlay_rect.min)
            .resizable(false)
            .collapsible(false)
            .show(ui.ctx(), |ui| {
                ui.image(&overlay_texture_handle);
            });

        map.overlay_texture = Some(overlay_texture_handle);
    }
}
