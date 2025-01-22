use eframe::egui;

use crate::app::{AppState, ViewMode};
use crate::app_impl::constants::SPACE;
use crate::grid::Grid;
use crate::lens::Lens;
use crate::movable::Draggable;
use crate::texture_request::TextureRequest;
use crate::tiles_behavior::MapsTreeBehavior;

impl AppState {
    fn show_tiles(&mut self, ui: &mut egui::Ui) {
        let mut behavior = MapsTreeBehavior {
            maps: &mut self.maps,
        };
        self.tile_manager.tree.ui(&mut behavior, ui);
    }

    fn show_stacked_images(&mut self, ui: &mut egui::Ui) {
        let num_visible = self.maps.values().filter(|m| m.visible).count();
        let rect_per_image = egui::Rect::from_min_max(
            egui::Pos2::ZERO,
            egui::pos2(
                ui.available_width(),
                ui.available_height() / num_visible as f32,
            ),
        );
        for (name, map) in self.maps.iter_mut() {
            if !map.visible {
                continue;
            }
            ui.with_layout(egui::Layout::top_down(egui::Align::TOP), |ui| {
                map.texture_state.put(
                    ui,
                    &TextureRequest::new(name.clone(), rect_per_image).with_tint(map.tint),
                );
            });
        }
    }

    fn show_grid(&mut self, ui: &mut egui::Ui) {
        // Modify the grid with the mouse, but only if inside this panel rect.
        let options = &mut self.options.grid;
        if ui.rect_contains_pointer(ui.available_rect_before_wrap()) {
            ui.input(|i| {
                if i.pointer.primary_down() {
                    // Scaled because meters are expected for drag().
                    options.drag(i.pointer.delta() / options.scale);
                }
                let scale_delta = i.smooth_scroll_delta.y * options.scroll_delta_percent;
                if !self.options.lens.enabled && scale_delta != 0. {
                    options.zoom(scale_delta);
                }
            });
        }

        let grid = Grid::new(ui, options.scale).with_origin_offset(options.offset);
        grid.show_maps(ui, &mut self.maps);
        if options.lines_visible {
            grid.draw(ui, options);
        }
        if options.marker_visible {
            grid.draw_axes(ui, options);
        }
        self.status.hover_position = grid.hover_pos_metric(ui);

        self.show_grid_lens(ui, self.status.hover_position);
    }

    pub fn show_grid_lens(&mut self, ui: &mut egui::Ui, center_pos: Option<egui::Pos2>) {
        let options = &self.options.grid;
        let grid_lens_scale = options.scale * 10.; // TODO: make this configurable.
        egui::Window::new(egui::RichText::new("🔍").strong())
            .title_bar(true)
            .auto_sized()
            .resizable(true)
            .collapsible(true)
            .default_size(egui::vec2(200., 200.))
            .default_pos(ui.clip_rect().min + egui::vec2(20., 20.))
            .show(ui.ctx(), |ui| {
                if let Some(center_pos) = center_pos {
                    let mini_grid = Grid::new(ui, grid_lens_scale).centered_at(center_pos);
                    mini_grid.show_maps(ui, &mut self.maps);
                    if options.lines_visible {
                        mini_grid.draw(ui, options);
                    }
                    if options.marker_visible {
                        mini_grid.draw_axes(ui, options);
                    }
                }
                // Fill window, grid is not a widget.
                ui.allocate_exact_size(ui.available_size(), egui::Sense::hover());
            });
    }

    fn show_lens(&mut self, ui: &mut egui::Ui) {
        let num_visible_maps = self.maps.values().filter(|m| m.visible).count();
        for (name, map) in &mut self.maps {
            if !self.options.lens.enabled {
                self.options.active_lens = None; // TODO: ensure this better.
                continue;
            }
            if self.options.view_mode == ViewMode::Aligned && num_visible_maps > 1 {
                // TODO: remove classic lens from aligned view mode.
                // Show lens on hover only for the active map in Aligned view mode.
                let active_lens = self.options.active_lens.get_or_insert(name.to_string());
                if *active_lens != *name {
                    continue;
                }
            }
            if Lens::with(&mut self.options.lens).show_on_hover(ui, map, name) {
                if self.options.view_mode != ViewMode::Aligned {
                    // TODO: Other modes don't set active_lens. Handle this better.
                    self.options.active_lens = Some(name.clone());
                }
            }
        }
    }

    fn show_empty(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(
            egui::Layout::centered_and_justified(egui::Direction::TopDown),
            |ui| {
                ui.horizontal_centered(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space((ui.available_height() / 2. - 100.).max(SPACE));
                        ui.heading("No maps loaded.");
                        ui.add_space(SPACE);
                        self.load_meta_button(ui);
                    });
                });
            },
        );
    }

    pub fn central_panel(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(self.options.canvas_settings.background_color))
            .show(ui.ctx(), |ui| {
                if self.maps.is_empty() {
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
                self.show_lens(ui);
            });
    }
}
