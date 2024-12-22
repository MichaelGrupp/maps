use std::collections::HashMap;
use std::default;
use std::option::Option;
use std::path::PathBuf;
use std::vec::Vec;

use eframe::egui;
use egui_file_dialog::FileDialog;
use image::GenericImageView;
use log::debug;

use crate::image::{fit_image, load_image, to_egui_image};
use crate::image_pyramid::ImagePyramid;
use crate::meta::Meta;

const SPACE: f32 = 10.;
const ICON_SIZE: f32 = 20.;

#[derive(Debug)]
struct AppOptions {
    menu_visible: bool,
    settings_visible: bool,
    desired_size: egui::Vec2,
    hover_region_size_meters: f32,
    hover_region_size_meters_min: f32,
    hover_region_size_meters_max: f32,
    hover_region_enabled: bool,
    scroll_speed_factor: f32,
}

impl default::Default for AppOptions {
    fn default() -> Self {
        AppOptions {
            menu_visible: false,
            settings_visible: false,
            desired_size: egui::vec2(0., 0.),
            hover_region_size_meters: 5.,
            hover_region_size_meters_min: 2.5,
            hover_region_size_meters_max: 25.,
            hover_region_enabled: true,
            scroll_speed_factor: 0.2,
        }
    }
}

#[derive(Default)]
struct TextureState {
    image_response: Option<egui::Response>,
    texture_handle: Option<egui::TextureHandle>,
}

struct MapState {
    meta: Meta,
    visible: bool,
    image_pyramid: ImagePyramid,
    texture_state: TextureState,
    overlay_texture: Option<egui::TextureHandle>,
}

#[derive(Default)]
pub struct AppState {
    options: AppOptions,
    maps: HashMap<String, MapState>,
    status_message: String,
    file_dialog: FileDialog,
}

#[derive(Debug)]
pub struct Error {
    pub message: String,
}

impl AppState {
    pub fn init(metas: Vec<Meta>) -> Result<AppState, Error> {
        let mut state = AppState::default();
        for meta in metas {
            state.load_image(meta)?;
        }
        Ok(state)
    }

    fn load_meta(&mut self, yaml_path: PathBuf) -> Result<bool, Error> {
        match Meta::load_from_file(yaml_path) {
            Ok(meta) => match self.load_image(meta) {
                Ok(_) => Ok(true),
                Err(e) => Err(e),
            },
            Err(e) => Err(Error {
                message: format!("Error loading metadata file: {:?}", e),
            }),
        }
    }

    fn load_meta_button(&mut self, ui: &mut egui::Ui) {
        if ui.button("📂 Load Map").clicked() {
            self.file_dialog.pick_file();
        }
        self.file_dialog.update(ui.ctx());

        if let Some(path) = self.file_dialog.take_picked() {
            match self.load_meta(path.clone()) {
                Ok(_) => {
                    self.status_message = format!("Loaded metadata file: {:?}", path);
                }
                Err(e) => {
                    self.status_message = format!("Error loading metadata file: {:?}", e.message);
                }
            }
        }
    }

    fn load_image(&mut self, meta: Meta) -> Result<(), Error> {
        self.status_message = format!("Loading image: {:?}", meta.image_path);
        match load_image(&meta.image_path) {
            Ok(image) => {
                let image_pyramid = ImagePyramid::new(image);
                self.maps.insert(
                    meta.image_path.to_str().unwrap().to_owned(),
                    MapState {
                        meta,
                        visible: true,
                        image_pyramid,
                        texture_state: TextureState::default(),
                        overlay_texture: None,
                    },
                );
                Ok(())
            }
            Err(e) => Err(Error {
                message: format!("Error loading image: {:?}", e),
            }),
        }
    }

    fn handle_key_shortcuts(&mut self, ui: &egui::Ui) {
        ui.input(|i| {
            if i.key_released(egui::Key::Escape) {
                self.options.menu_visible = false;
                self.options.settings_visible = false;
                self.options.hover_region_enabled = false;
            } else if i.key_released(egui::Key::R) {
                self.options.hover_region_enabled = !self.options.hover_region_enabled;
            }
            if i.key_released(egui::Key::M) {
                self.options.menu_visible = !self.options.menu_visible;
            }
            if i.key_released(egui::Key::S) {
                self.options.settings_visible = !self.options.settings_visible;
            }
        });
    }

    fn update_desired_size(&mut self, ui: &egui::Ui) {
        let old_size = self.options.desired_size;
        let pixels_per_point = ui.ctx().zoom_factor() * ui.ctx().pixels_per_point();
        let desired_size = egui::vec2(
            ui.available_width() * pixels_per_point,
            ui.available_height() * pixels_per_point,
        );
        if desired_size != old_size {
            // Note that in egui dropping the last handle of a texture will free it.
            debug!(
                "Desired size changed to {:?}, clearing textures.",
                desired_size
            );
            for map in self.maps.values_mut() {
                map.texture_state.texture_handle = None;
            }
        }
        self.options.desired_size = desired_size;
    }

    fn update_texture_handles(&mut self, ui: &egui::Ui) {
        self.update_desired_size(ui);

        for (name, map) in &mut self.maps {
            map.texture_state.texture_handle.get_or_insert_with(|| {
                // Load the texture only if needed.
                debug!("Fitting and reloading texture for: {}", name);
                let image_pyramid = &map.image_pyramid;
                ui.ctx().load_texture(
                    name,
                    to_egui_image(fit_image(
                        image_pyramid
                            .get_level(self.options.desired_size.max_elem() as u32)
                            .clone(),
                        self.options.desired_size,
                    )),
                    Default::default(),
                )
            });
        }
    }

    fn show_images(&mut self, ui: &mut egui::Ui) {
        self.update_texture_handles(ui);

        let options = &mut self.options;
        for (name, map) in self.maps.iter_mut() {
            if !map.visible {
                continue;
            }
            ui.with_layout(egui::Layout::top_down(egui::Align::TOP), |ui| {
                Self::show_image(ui, name, map);
                Self::show_overlay(ui, name, map, options);
            });
        }
    }

    fn show_image(ui: &mut egui::Ui, name: &str, map: &mut MapState) {
        let texture = match &map.texture_state.texture_handle {
            Some(texture) => texture,
            None => {
                panic!("Missing texture handle for image {}", name);
            }
        };
        map.texture_state.image_response = Some(ui.image(texture));
    }

    fn show_overlay(ui: &mut egui::Ui, name: &str, map: &mut MapState, options: &mut AppOptions) {
        if !options.hover_region_enabled {
            return;
        }

        let response = match &map.texture_state.image_response {
            Some(response) => response,
            None => {
                panic!("Missing image response for image {}", name);
            }
        };

        let Some(pointer_pos) = response.hover_pos() else {
            // Clear the overlay texture if the mouse is not hovering over the image.
            map.overlay_texture = None;
            return;
        };

        ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);

        // Change the hover region size when scrolling.
        options.hover_region_size_meters = (options.hover_region_size_meters
            + ui.input(|i| i.smooth_scroll_delta).y * options.scroll_speed_factor)
            .clamp(
                options.hover_region_size_meters_min,
                options.hover_region_size_meters_max,
            );

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
        let hover_region_size_pixels =
            options.hover_region_size_meters / map.meta.resolution as f32;
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

    fn header_panel(&mut self, ui: &mut egui::Ui) {
        let add_toggle_button = |ui: &mut egui::Ui,
                                 icon: &str,
                                 tooltip: &str,
                                 switch: &mut bool| {
            if ui
                .add_sized(
                    egui::vec2(ICON_SIZE, ICON_SIZE),
                    egui::SelectableLabel::new(*switch, egui::RichText::new(icon).size(ICON_SIZE)),
                )
                .on_hover_text(tooltip)
                .clicked()
            {
                *switch = !*switch;
            }
        };

        egui::TopBottomPanel::new(egui::containers::panel::TopBottomSide::Top, "header").show(
            ui.ctx(),
            |ui| {
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                        add_toggle_button(ui, "☰", "Show Menu", &mut self.options.menu_visible);
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        add_toggle_button(
                            ui,
                            "⚙",
                            "Show Settings",
                            &mut self.options.settings_visible,
                        );
                    });
                });
            },
        );
    }

    fn menu_panel(&mut self, ui: &mut egui::Ui) {
        if !self.options.menu_visible {
            return;
        }
        egui::SidePanel::left("menu").show(ui.ctx(), |ui| {
            ui.heading("Maps");
            ui.add_space(SPACE);
            self.load_meta_button(ui);
            ui.separator();
            for (name, map) in &mut self.maps {
                ui.checkbox(&mut map.visible, name);
            }
        });
    }

    fn settings_panel(&mut self, ui: &mut egui::Ui) {
        if !self.options.settings_visible {
            return;
        }
        egui::SidePanel::right("settings").show(ui.ctx(), |ui| {
            ui.heading("Settings");
            ui.add_space(SPACE);
            ui.checkbox(&mut self.options.hover_region_enabled, "Show ROI");
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                ui.label("ROI size (meters):");
                ui.add(egui::Slider::new(
                    &mut self.options.hover_region_size_meters,
                    self.options.hover_region_size_meters_min
                        ..=self.options.hover_region_size_meters_max,
                ));
            });
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                ui.label("Scroll speed factor:");
                ui.add(egui::Slider::new(
                    &mut self.options.scroll_speed_factor,
                    0.0..=1.0,
                ));
            });
        });
    }

    fn footer_panel(&mut self, ui: &mut egui::Ui) {
        egui::TopBottomPanel::new(egui::containers::panel::TopBottomSide::Bottom, "footer").show(
            ui.ctx(),
            |ui| {
                ui.horizontal(|ui| ui.label(self.status_message.clone()));
            },
        );
    }

    fn central_panel(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui.ctx(), |ui| {
            if self.maps.is_empty() {
                ui.with_layout(
                    egui::Layout::centered_and_justified(egui::Direction::TopDown),
                    |ui| {
                        ui.vertical_centered(|ui| {
                            ui.heading("No maps loaded.");
                            ui.add_space(SPACE);
                            self.load_meta_button(ui);
                        });
                    },
                );
                return;
            }

            egui::ScrollArea::both().show(ui, |ui| {
                self.show_images(ui);
                // Fill the remaining vertical space, otherwise the scroll bar can jump around.
                ui.add_space(ui.available_height());
            });
        });
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.handle_key_shortcuts(ui);

            self.header_panel(ui);
            self.menu_panel(ui);
            self.settings_panel(ui);
            self.central_panel(ui);
            self.footer_panel(ui);
        });
    }
}
