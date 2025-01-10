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
                    options.drag(i.pointer.delta());
                }
                let scale_delta = i.smooth_scroll_delta.y * options.scroll_speed_factor;
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
            grid.draw_axes(ui, &options);
        }
    }

    fn show_lens(&mut self, ui: &mut egui::Ui) {
        let num_visible_maps = self.maps.values().filter(|m| m.visible).count();
        for (name, map) in &mut self.maps {
            if !self.options.lens.enabled {
                continue;
            }
            if self.options.view_mode == ViewMode::Aligned && num_visible_maps > 1 {
                // Show lens on hover only for the active map in Aligned view mode.
                let active_lens = self.options.active_lens.get_or_insert(name.to_string());
                if *active_lens != *name {
                    continue;
                }
            }
            if Lens::with(&mut self.options.lens).show_on_hover(ui, map, name) {
                self.status_message = format!("Lens active on: {}", name);
                return;
            }
        }
        // TODO: use separate status message for lens.
        self.status_message = "".to_string();
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
