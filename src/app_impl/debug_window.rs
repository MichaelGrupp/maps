use eframe::egui;

use crate::app::AppState;

impl AppState {
    pub(crate) fn debug_window(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        if !self.status.debug_window_active {
            return;
        }
        egui::Window::new("Debug")
            .open(&mut self.status.debug_window_active)
            .frame(egui::Frame::canvas(ui.style()).multiply_with_opacity(0.75))
            .show(ui.ctx(), |ui| {
                egui::ScrollArea::both().show(ui, |ui| {
                    ui.collapsing("Settings", |ui| {
                        ctx.settings_ui(ui);
                    });
                    ui.collapsing("Inspection", |ui| {
                        ctx.inspection_ui(ui);
                    });
                    ui.collapsing("Memory", |ui| {
                        ctx.memory_ui(ui);
                    });
                    egui::CollapsingHeader::new("Timing")
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.label(format!(
                                "Last {} {} durations in seconds",
                                self.tracing.buffer_size(),
                                self.tracing.name.as_str()
                            ));
                            self.tracing.plot(ui);
                        });
                    egui::CollapsingHeader::new("Textures")
                        .default_open(true)
                        .show(ui, |ui| {
                            ctx.texture_ui(ui);
                        });
                });
            });
    }
}
