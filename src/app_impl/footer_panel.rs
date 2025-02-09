use eframe::egui;
use log::log_enabled;

use crate::app::{AppState, ViewMode};

impl AppState {
    pub fn footer_panel(&mut self, ui: &mut egui::Ui) {
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
                    });
                });
            },
        );
    }
}
