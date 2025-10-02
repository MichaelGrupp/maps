use eframe::egui;
use log::error;

use crate::app::AppState;
use crate::app_impl::constants::SPACE;
use crate::app_impl::ui_helpers::display_path;
use crate::map_state::MapState;
use maps_rendering::TextureRequest;

fn map_tooltip(ui: &mut egui::Ui, name: &str, map: &mut MapState) {
    ui.horizontal_top(|ui| {
        map.get_or_create_texture_state("tooltip_thumbnail").put(
            ui,
            &TextureRequest::new(
                "tooltip".to_owned(),
                egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(200.)),
            ),
        );
        egui::Grid::new("map_info")
            .striped(true)
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("YAML file path");
                ui.label(name);
                ui.end_row();
                ui.label("Image file path");
                ui.label(map.meta.image_path.to_str().expect("invalid path?"));
                ui.end_row();
                ui.label("Image file size");
                ui.label(format!(
                    "{} x {} pixels",
                    map.image_pyramid.original_size.x, map.image_pyramid.original_size.y
                ));
                ui.end_row();
                ui.label("Resolution");
                ui.label(format!("{} m/pixel", map.meta.resolution));
            });
    });
}

impl AppState {
    pub(crate) fn menu_panel(&mut self, ui: &mut egui::Ui) {
        if !self.options.menu_visible {
            return;
        }
        egui::SidePanel::left("menu").show(ui.ctx(), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                self.menu_content(ui);
            });
        });
    }

    fn deselect_toggle(&mut self, ui: &mut egui::Ui) {
        if self.data.maps.is_empty() {
            return;
        }
        let mut all_off = self.data.maps.iter().all(|(_, map)| !map.visible);
        let icon = if all_off { "☑" } else { "⛶" };
        let action = if all_off { "Select" } else { "Deselect" };
        // Toggle value is less obtrusive than a button / check box.
        // Only highlighted if all are off.
        if ui
            .toggle_value(&mut all_off, icon)
            .on_hover_text(format!("{action} all."))
            .clicked()
        {
            for (name, map) in &mut self.data.maps {
                map.visible = !all_off;
                self.tile_manager.set_visible(name, !all_off);
            }
        }
    }

    fn maps_list(&mut self, ui: &mut egui::Ui) {
        let mut to_delete: Vec<String> = Vec::new();
        egui::Grid::new("maps_list")
            .num_columns(3)
            .striped(true)
            .show(ui, |ui| {
                for name in self.data.draw_order.keys() {
                    let Some(map) = self.data.maps.get_mut(name) else {
                        error!("Unknown draw order key: {name}");
                        continue;
                    };

                    if ui
                        .checkbox(
                            &mut map.visible,
                            display_path(name, self.options.display.show_full_paths),
                        )
                        .on_hover_ui(|ui| {
                            map_tooltip(ui, name, map);
                        })
                        .changed()
                    {
                        self.tile_manager.set_visible(name, map.visible);
                    }
                    if ui.button("🗑").on_hover_text("Delete Map").clicked() {
                        to_delete.push(name.clone());
                    }
                    if map.meta.origin_theta.angle() != 0. {
                        ui.label(
                            egui::RichText::new("⚠")
                                .strong()
                                .color(egui::Color32::ORANGE),
                        )
                        .on_hover_text(
                            "This map has a non-zero origin rotation component in its metadata.\n\
                            maps uses it, but it's not supported by most ROS tools.\n\n\
                            It's recommended to save alignment transformations separately,\n\
                            e.g. using the Pose editor here.",
                        );
                    }
                    ui.end_row();
                }
            });
        self.delete(&to_delete);
    }

    fn menu_content(&mut self, ui: &mut egui::Ui) {
        ui.heading("Maps");
        ui.add_space(SPACE);
        ui.horizontal(|ui| {
            self.load_meta_button(ui);
            ui.separator();
            #[cfg(not(target_arch = "wasm32"))]
            self.load_session_button(ui);
            #[cfg(target_arch = "wasm32")]
            ui.add_enabled_ui(false, |ui| {
                self.load_session_button(ui);
            });
            #[cfg(not(target_arch = "wasm32"))]
            self.save_session_button(ui, false);
            #[cfg(target_arch = "wasm32")]
            ui.add_enabled_ui(false, |ui| {
                self.save_session_button(ui, false);
            });
        });
        ui.separator();

        // Allow to hide list to resize panel smaller, e.g. with long paths.
        egui::CollapsingHeader::new("List")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.toggle_value(&mut self.status.draw_order_edit_active, "⬆⬇")
                        .on_hover_text("Click to edit the draw order via drag and drop.");
                    if !self.status.draw_order_edit_active && !self.data.maps.is_empty() {
                        ui.separator();
                        self.deselect_toggle(ui);
                    }
                    ui.separator();
                    ui.toggle_value(&mut self.options.display.show_full_paths, "/../..")
                        .on_hover_text("Show full paths instead of just the file name.");
                });
                if self.status.draw_order_edit_active {
                    self.data
                        .draw_order
                        .ui(ui, self.options.display.show_full_paths);
                } else {
                    self.maps_list(ui);
                }
            });

        if self.data.maps.is_empty() {
            return;
        }

        ui.separator();
        ui.add_space(SPACE);
        ui.heading("Pose");
        ui.add_space(SPACE);
        self.pose_edit(ui);
        if !self.options.pose_edit.selected_map.is_empty() && self.data.maps.len() > 1 {
            ui.separator();
            ui.add_space(SPACE);
            egui::ScrollArea::horizontal().show(ui, |ui| {
                // In scroll area to not take too much space for long paths.
                self.apply_pose_to_other_maps(ui);
            });
        }
    }
}
