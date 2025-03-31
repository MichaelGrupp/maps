use std::default;

use eframe::egui;
use log::debug;
use serde::{Deserialize, Serialize};

use crate::app::CanvasOptions;
use crate::image::{color_to_alpha, to_egui_image};
use crate::map_state::MapState;
use crate::texture_state::TextureState;

#[derive(Debug, Serialize, Deserialize)]
pub struct LensOptions {
    pub size_meters: f32,
    pub size_meters_min: f32,
    pub size_meters_max: f32,
    pub scroll_speed_factor: f32,
}

impl default::Default for LensOptions {
    fn default() -> LensOptions {
        LensOptions {
            size_meters: 5.,
            size_meters_min: 2.5,
            size_meters_max: 25.,
            scroll_speed_factor: 0.2,
        }
    }
}

pub struct Lens<'a> {
    // Options are mutably borrowed with outer lifetime
    // to allow managing them outside.
    options: &'a mut LensOptions,
}

impl<'a> Lens<'a> {
    pub fn with(options: &'a mut LensOptions) -> Lens<'a> {
        Lens { options }
    }

    pub fn show_on_hover(
        &mut self,
        ui: &mut egui::Ui,
        map: &mut MapState,
        texture_state_id: &str,
        canvas_settings: &CanvasOptions,
    ) -> bool {
        let options = &mut self.options;

        let texture_state = map
            .texture_states
            .entry(texture_state_id.to_string())
            .or_insert(TextureState::new(map.image_pyramid.clone()));

        let response = match &texture_state.image_response {
            Some(response) => response,
            None => {
                // Can be missing e.g. if a tab is not visible yet.
                return false;
            }
        };

        let Some(pointer_pos) = response.hover_pos() else {
            return false;
        };

        ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);

        // Change the hover region size when scrolling.
        options.size_meters = (options.size_meters
            + ui.input(|i| i.smooth_scroll_delta).y * options.scroll_speed_factor)
            .clamp(options.size_meters_min, options.size_meters_max);

        // Show an overlay with a crop region of the original size image.
        // For this, the pointer position in the rendered texture needs to be converted
        // to corresponding coordinates in the unscaled original image.

        // UV coordinates in the visible texture.
        let texture_size = &response.rect.size();
        let texture_pos = pointer_pos - response.rect.min;
        let lens_uv = egui::vec2(
            texture_pos.x / texture_size.x,
            texture_pos.y / texture_size.y,
        );

        // When partially visible, we deal with a UV rect inside an UV rect.
        let texture_uv = texture_state.desired_uv;

        let original_image = &texture_state.image_pyramid.original;
        let original_width = original_image.width() as f32;
        let original_height = original_image.height() as f32;
        let crop_width = original_width * (texture_uv[1].x - texture_uv[0].x);
        let crop_height = original_height * (texture_uv[1].y - texture_uv[0].y);
        let original_pos = egui::vec2(
            texture_uv[0].x * original_width + lens_uv.x * crop_width,
            texture_uv[0].y * original_height + lens_uv.y * crop_height,
        );

        // Get crop for the overlay. The result can be smaller at the border.
        let region_size_pixels = options.size_meters / map.meta.resolution;
        let half_region_size = region_size_pixels / 2.;
        let min_x = (original_pos.x - half_region_size).max(0.) as u32;
        let min_y = (original_pos.y - half_region_size).max(0.) as u32;
        let max_x = (original_pos.x + half_region_size).min(original_width) as u32;
        let max_y = (original_pos.y + half_region_size).min(original_height) as u32;
        if min_x >= max_x || min_y >= max_y {
            debug!("Ignoring hover because region would be empty.");
            return false;
        }
        let mut cropped_image = original_image.crop_imm(min_x, min_y, max_x - min_x, max_y - min_y);
        color_to_alpha(&mut cropped_image, map.color_to_alpha);
        if map.use_value_interpretation {
            map.meta.value_interpretation.apply(
                &mut cropped_image,
                texture_state.image_pyramid.original_has_alpha,
            );
        }
        let cropped_size = egui::vec2(cropped_image.width() as f32, cropped_image.height() as f32);

        let overlay_texture_handle = ui.ctx().load_texture(
            "overlay_".to_owned() + texture_state_id,
            to_egui_image(cropped_image),
            Default::default(),
        );

        // Show the crop area also in the scaled texture coordinates as a small rectangle.
        let small_rect_ratio = texture_size.x / original_width;
        self.lens_rect(
            ui,
            egui::Rect::from_min_size(
                // Clamp to the texture bounds to show correctly at borders.
                (pointer_pos - (cropped_size * small_rect_ratio) / 2.)
                    .max(response.rect.min)
                    .min(response.rect.max - cropped_size * small_rect_ratio),
                cropped_size * small_rect_ratio,
            ),
        );

        // Show overlay in diagonally opposite direction of the hovered quadrant.
        let overlay_pos = Self::bounce_pos(ui, pointer_pos, cropped_size);
        let overlay_rect = egui::Rect::from_center_size(overlay_pos, cropped_size);

        // Draw rectangle around the overlay, a bit wider than the overlay itself.
        let stroke = egui::Stroke::new(5., egui::Rgba::from_rgb(0., 0., 0.));
        ui.painter().add(egui::Shape::rect_filled(
            overlay_rect,
            1.,
            ui.visuals().extreme_bg_color,
        ));
        ui.painter().add(egui::Shape::rect_stroke(
            overlay_rect,
            1.,
            stroke,
            egui::StrokeKind::Outside,
        ));

        // Ensure that the lens has the same background color as the canvas.
        // (e.g. if color_to_alpha is used)
        ui.painter()
            .rect_filled(overlay_rect, 0., canvas_settings.background_color);
        // TODO: use TextureRequest to load the overlay image.
        ui.put(
            overlay_rect,
            egui::Image::new(&overlay_texture_handle)
                .tint(map.tint.unwrap_or(egui::Color32::WHITE)),
        );

        true
    }

    fn lens_rect(&mut self, ui: &egui::Ui, rect: egui::Rect) {
        let stroke = egui::Stroke::new(2., egui::Rgba::from_rgb(0., 0., 0.));
        let fill = egui::Rgba::from_black_alpha(0.25);
        ui.painter().add(egui::Shape::rect_filled(rect, 0., fill));
        ui.painter().add(egui::Shape::rect_stroke(
            rect,
            1.,
            stroke,
            egui::StrokeKind::Middle,
        ));
    }

    fn bounce_pos(ui: &egui::Ui, pointer_pos: egui::Pos2, overlay_size: egui::Vec2) -> egui::Pos2 {
        let offset = overlay_size / 2. + egui::vec2(10., 10.);
        let window_uv = egui::vec2(
            pointer_pos.x / ui.ctx().screen_rect().width(),
            pointer_pos.y / ui.ctx().screen_rect().height(),
        );

        if window_uv.x < 0.5 && window_uv.y < 0.5 {
            pointer_pos + offset
        } else if window_uv.x < 0.5 {
            pointer_pos + offset * egui::vec2(1., -1.)
        } else if window_uv.y < 0.5 {
            pointer_pos + offset * egui::vec2(-1., 1.)
        } else {
            pointer_pos - offset
        }
    }
}
