use eframe::egui;
use log::{debug, error};
use uuid::Uuid;

use crate::app::{ActiveTool, AppState, ViewMode};
use crate::app_impl::constants::SPACE;
use crate::grid::Grid;
use crate::grid_options::{LineType, SubLineVisibility};
use crate::lens::Lens;
use crate::texture_request::TextureRequest;
use crate::tiles_behavior::MapsTreeBehavior;

const STACKED_TEXTURE_ID: &str = "stack";

impl AppState {
    fn show_tiles(&mut self, ui: &mut egui::Ui) {
        let hovered_id = {
            let mut behavior = MapsTreeBehavior {
                maps: &mut self.data.maps,
                hovered_id: None,
            };
            self.tile_manager.tree.ui(&mut behavior, ui);
            behavior.hovered_id
        };

        if let Some(hovered_id) = hovered_id {
            self.show_lens(ui, &hovered_id, &hovered_id);
        } else {
            self.status.active_tool = None;
        }
    }

    fn show_stacked_images(&mut self, ui: &mut egui::Ui) {
        let num_visible = self.data.maps.values().filter(|m| m.visible).count();
        let rect_per_image = egui::Rect::from_min_max(
            egui::Pos2::ZERO,
            egui::pos2(
                ui.available_width(),
                ui.available_height() / num_visible as f32,
            ) * self.options.canvas_settings.stack_scale_factor,
        );
        self.status.active_tool = None;
        for name in self.data.draw_order.keys() {
            let Some(map) = self.data.maps.get_mut(name) else {
                error!("Unknown draw order key: {}", name);
                continue;
            };

            if !map.visible {
                continue;
            }
            ui.with_layout(egui::Layout::top_down(egui::Align::TOP), |ui| {
                let request = &TextureRequest::new(name.clone(), rect_per_image)
                    .with_tint(map.tint)
                    .with_color_to_alpha(map.color_to_alpha)
                    .with_thresholding(map.get_value_interpretation())
                    .with_texture_options(&map.texture_filter.get(1.));
                map.get_or_create_texture_state(STACKED_TEXTURE_ID)
                    .put(ui, request);
                if let Some(response) = &map
                    .get_or_create_texture_state(STACKED_TEXTURE_ID)
                    .image_response
                {
                    if response.hovered() {
                        self.status.active_tool = Some(name.clone());
                    }
                }
            });
        }
        if let Some(hovered_map) = &self.status.active_tool {
            self.show_lens(ui, hovered_map.clone().as_str(), STACKED_TEXTURE_ID);
        }
    }

    fn show_grid(&mut self, ui: &mut egui::Ui) {
        let options = &mut self.options.grid;

        let grid = Grid::new(ui, "main_grid", options.scale)
            .with_origin_offset(options.offset)
            .with_texture_crop_threshold(self.options.advanced.grid_crop_threshold);

        // Handle input interaction and adapt mouse pointer to the active tool.
        if grid.response().hovered() {
            match self.options.active_tool {
                ActiveTool::PlaceLens | ActiveTool::Measure | ActiveTool::HoverLens => {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
                }
                _ => {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Default);
                }
            }
        }
        // Note: the updated grid options are used in the next frame.
        grid.update_drag_and_zoom(ui, options);

        grid.show_maps(ui, &mut self.data.maps, options, &self.data.draw_order);
        if options.lines_visible {
            grid.draw_lines(options, LineType::Main);
        }
        if options.sub_lines_visible == SubLineVisibility::Always {
            grid.draw_lines(options, LineType::Sub);
        }
        if options.marker_visibility.zero_visible() {
            grid.draw_axes(options, None);
        }
        self.status.hover_position = grid.hover_pos_metric();

        if self.options.active_tool == ActiveTool::None {
            self.status.active_tool = None;
        }
        if self.options.active_tool == ActiveTool::HoverLens {
            self.status.active_tool =
                Some(format!("🔍 {}x magnification", options.lens_magnification));
        }
        if self.options.active_tool == ActiveTool::HoverLens {
            self.show_grid_lens(ui, self.status.hover_position, "hover_lens", false, None);
            // Don't show the other fixed lenses too not get too messy.
            return;
        }

        if self.options.active_tool == ActiveTool::Measure {
            self.status.active_tool = Some("📏 Measurement tool active".to_string());
            if !grid.response().clicked() {
                grid.draw_measure(options, self.status.hover_position);
                return;
            }
            if let Some(click_pos) = self.status.hover_position {
                if options.measure_start.is_none() {
                    options.measure_start = Some(click_pos);
                } else if options.measure_end.is_none() {
                    options.measure_end = Some(click_pos);
                } else {
                    options.measure_start = Some(click_pos);
                    options.measure_end = None;
                }
            }
            // Don't show fixed lenses when measuring.
            return;
        }

        if grid.response().clicked() && self.options.active_tool == ActiveTool::PlaceLens {
            if let Some(pos) = self.status.hover_position {
                let id = Uuid::new_v4().to_string();
                debug!("Placing lens {} focussing {:?}.", id, pos);
                self.data.grid_lenses.insert(id, pos);
                self.status.unsaved_changes = true;
                self.options.active_tool = ActiveTool::None;
            }
        }
        let lens_ids = self.data.grid_lenses.keys().cloned().collect::<Vec<_>>();
        if self.options.active_tool == ActiveTool::PlaceLens || !lens_ids.is_empty() {
            self.status.active_tool = Some(format!(
                "🔍 {} fixed lenses active at {}x magnification",
                self.data.grid_lenses.len(),
                options.lens_magnification
            ));
        }
        for (i, lens_id) in lens_ids.iter().enumerate() {
            if let Some(pos) = self.data.grid_lenses.get(lens_id) {
                self.show_grid_lens(
                    ui,
                    Some(*pos),
                    lens_id.clone().as_str(),
                    true,
                    // Offset each new lens window a bit.
                    Some(i as f32 * egui::vec2(20., 20.)),
                );
            }
        }
    }

    pub(crate) fn show_grid_lens(
        &mut self,
        ui: &mut egui::Ui,
        center_pos: Option<egui::Pos2>,
        id: &str,
        closable: bool,
        default_offset: Option<egui::Vec2>,
    ) {
        let options = &self.options.grid;
        let grid_lens_scale = options.scale * options.lens_magnification;
        let mut open = true;
        let mut window = egui::Window::new(egui::RichText::new("🔍").strong())
            .title_bar(true)
            .id(egui::Id::new(id))
            .auto_sized()
            .resizable(true)
            .collapsible(true)
            .default_size(egui::vec2(250., 250.))
            .default_pos(ui.clip_rect().min + default_offset.unwrap_or(egui::vec2(0., 0.)));
        if closable {
            window = window.open(&mut open);
        }
        window.show(ui.ctx(), |ui| {
            // Show the lens grid.
            // Crop threshold is set to 0 to always crop the textures in a lens.
            let mini_grid = Grid::new(ui, id, grid_lens_scale)
                .centered_at(center_pos.unwrap_or_default())
                .with_texture_crop_threshold(0);
            // Always fill the lens window with a background rectangle.
            // Ensure that the lens uses the same background color as the main grid canvas.
            mini_grid.draw_background(self.options.canvas_settings.background_color);
            // Only show actual data if the center is set (can be None when hover lens loses focus).
            if center_pos.is_some() {
                mini_grid.show_maps(ui, &mut self.data.maps, options, &self.data.draw_order);
                if options.lines_visible {
                    mini_grid.draw_lines(options, LineType::Main);
                }
                if options.sub_lines_visible == SubLineVisibility::Always
                    || options.sub_lines_visible == SubLineVisibility::OnlyLens
                {
                    mini_grid.draw_lines(options, LineType::Sub);
                }
                if options.marker_visibility.zero_visible() {
                    mini_grid.draw_axes(options, None);
                }
            }
        });
        if !open {
            self.data.grid_lenses.remove(id);
            for (name, map) in self.data.maps.iter_mut() {
                debug!(
                    "Removing lens texture state with ID {} from map {}.",
                    id, name
                );
                map.texture_states.remove(id);
            }
        }
    }

    fn show_lens(&mut self, ui: &mut egui::Ui, map_id: &str, texture_id: &str) {
        if self.options.view_mode == ViewMode::Aligned {
            // The "classic" lens is not shown in aligned mode, we add grids there.
            return;
        }
        if self.options.active_tool != ActiveTool::HoverLens {
            self.status.active_tool = None;
            return;
        }

        if let Some(map) = self.data.maps.get_mut(map_id) {
            if Lens::with(&mut self.options.lens).show_on_hover(
                ui,
                map,
                texture_id,
                &self.options.canvas_settings,
            ) && self.options.view_mode != ViewMode::Aligned
            {
                self.status.active_tool = Some(map_id.to_string());
            }
        }
    }

    fn show_empty(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(
            egui::Layout::centered_and_justified(egui::Direction::TopDown),
            |ui| {
                ui.horizontal_centered(|ui| {
                    ui.vertical_centered(|ui| {
                        let frac = if cfg!(target_arch = "wasm32") { 4. } else { 2. };
                        ui.add_space((ui.available_height() / frac - 100.).max(SPACE));
                        ui.heading("No maps loaded.");
                        ui.add_space(SPACE);
                        ui.add_space(SPACE);
                        self.load_meta_button(ui);
                        ui.add_space(SPACE);

                        #[cfg(not(target_arch = "wasm32"))]
                        self.load_session_button(ui);
                        #[cfg(target_arch = "wasm32")]
                        ui.add_enabled_ui(false, |ui| {
                            self.load_session_button(ui);
                        });

                        #[cfg(target_arch = "wasm32")]
                        {
                            ui.add_space(SPACE * 3.);
                            ui.label(
                                egui::RichText::new(
                                    "Filesystem IO is limited in the web assembly app.",
                                )
                                .color(egui::Color32::ORANGE),
                            );
                            ui.add(
                                egui::Hyperlink::from_label_and_url(
                                    "Click here to learn more.",
                                    "https://github.com/MichaelGrupp/maps?tab=readme-ov-file#maps",
                                )
                                .open_in_new_tab(true),
                            );
                            ui.add_space(SPACE);
                            self.demo_buttons(ui);
                        }
                    });
                });
            },
        );
    }

    /// Central panel that shows the map content.
    /// Returns the rect of the viewport for screenshot purposes.
    pub(crate) fn central_panel(&mut self, ui: &mut egui::Ui) -> egui::Rect {
        let mut viewport_rect = egui::Rect::ZERO;

        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(self.options.canvas_settings.background_color))
            .show(ui.ctx(), |ui| {
                viewport_rect = ui.clip_rect();

                if self.data.maps.is_empty() {
                    self.show_empty(ui);
                    return;
                }

                match self.options.view_mode {
                    ViewMode::Tiles => {
                        self.show_tiles(ui);
                    }
                    ViewMode::Stacked => {
                        egui::ScrollArea::both().show(ui, |ui| {
                            self.show_stacked_images(ui);
                            // Fill the remaining vertical space, otherwise the scroll bar can jump around.
                            ui.add_space(ui.available_height());
                        });
                    }
                    ViewMode::Aligned => {
                        self.show_grid(ui);
                    }
                }
            });

        viewport_rect
    }
}
