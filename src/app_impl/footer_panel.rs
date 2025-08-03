use eframe::egui;
use log::log_enabled;

use crate::app::{ActiveMovable, AppState, ViewMode};

impl AppState {
    fn select_movable(&mut self, ui: &mut egui::Ui) {
        if self.options.active_movable == ActiveMovable::MapPose {
            // Show a combo box to select the map for pose editing.
            self.pose_edit_combo_box(ui);
        }
        // Show combo box to select the active movable.
        egui::ComboBox::from_label("Movable:")
            .width(0.)  // Fit as small as possible.
            .selected_text(self.options.active_movable.to_string())
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.options.active_movable,
                    ActiveMovable::MapPose,
                    "Map Pose",
                );
                ui.selectable_value(
                    &mut self.options.active_movable,
                    ActiveMovable::Grid,
                    "Grid",
                );
            })
            .response
            .on_hover_text("Select what can be moved by the WASD/QE keys.");
    }

    pub(crate) fn footer_panel(&mut self, ui: &mut egui::Ui) {
        egui::TopBottomPanel::new(egui::containers::panel::TopBottomSide::Bottom, "footer").show(
            ui.ctx(),
            |ui| {
                ui.horizontal(|ui| {
                    if let Some(active_tool) = self.status.active_tool.as_ref() {
                        if self.options.view_mode == ViewMode::Aligned {
                            ui.label(active_tool).on_hover_text(
                                "Magnification can be changed in the options side menu.",
                            );
                        } else {
                            ui.label(format!(
                                "🔍 ({:.1}m) {}",
                                self.options.lens.size_meters, active_tool
                            ));
                        }
                        ui.separator();
                    }
                    if let Some(pos) = self.status.hover_position {
                        ui.label(format!("⌖ x: {:.3}m  y: {:.3}m", pos.x, pos.y,));
                        ui.separator();
                    }
                    if let Some(move_action) = &self.status.move_action {
                        if self.options.view_mode == ViewMode::Aligned {
                            ui.label(move_action);
                            ui.separator();
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(egui::Button::new("ℹ").fill(ui.visuals().window_fill()))
                            .on_hover_text("Open the information window.")
                            .clicked()
                        {
                            self.options.help_visible = !self.options.help_visible;
                        }
                        ui.separator();
                        if log_enabled!(log::Level::Debug) {
                            if ui
                                .add(egui::Button::new("🛠").fill(ui.visuals().window_fill()))
                                .on_hover_text("Open the debug window.")
                                .clicked()
                            {
                                self.status.debug_window_active = !self.status.debug_window_active;
                            }
                            ui.separator();
                        }
                        ui.scope(|ui| {
                            // Fill combo box also with dark color to fit the style of the footer panel.
                            ui.visuals_mut().widgets.inactive.weak_bg_fill =
                                ui.visuals().window_fill();
                            if self.options.view_mode == ViewMode::Aligned {
                                self.select_movable(ui);
                                ui.separator();
                            }
                        });
                        egui::warn_if_debug_build(ui);
                    });
                });
            },
        );
    }
}
