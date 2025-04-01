use eframe::egui;

use crate::app::AppState;
use crate::app_impl::constants::SPACE;

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
                for (name, map) in &mut self.data.maps {
                    if ui.checkbox(&mut map.visible, name).changed() {
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
                        .on_hover_text("Click to view and edit the draw order via drag and drop.");
                    if !self.status.draw_order_edit_active {
                        ui.separator();
                        self.deselect_toggle(ui);
                    }
                });
                if self.status.draw_order_edit_active {
                    self.data.draw_order.ui(ui);
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
