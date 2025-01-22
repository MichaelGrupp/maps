use eframe::egui;

use crate::app::AppState;

impl AppState {
    pub fn footer_panel(&mut self, ui: &mut egui::Ui) {
        egui::TopBottomPanel::new(egui::containers::panel::TopBottomSide::Bottom, "footer").show(
            ui.ctx(),
            |ui| {
                ui.horizontal(|ui| {
                    if let Some(pos) = self.status.hover_position {
                        ui.label(format!("x: {:.3}m  y: {:.3}m", pos.x, pos.y,));
                        ui.separator();
                    }
                    if let Some(active_lens) = self.options.active_lens.as_ref() {
                        ui.label(format!(
                            "🔍 ({:.1}m) {}",
                            self.options.lens.size_meters, active_lens
                        ));
                        ui.separator();
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
                    });
                });
            },
        );
    }
}
