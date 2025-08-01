use eframe::egui;

use crate::app::AppState;
use crate::app_impl::ui_helpers::section_heading;
use crate::lens::LensOptions;

impl AppState {
    pub(crate) fn lens_settings(&mut self, ui: &mut egui::Ui) {
        section_heading(ui, "Lens", &mut self.options.collapsed.lens_settings);
        if ui.button("Reset").clicked() {
            self.options.lens = LensOptions::default();
        }
        if self.options.collapsed.lens_settings {
            return;
        }
        ui.end_row();
        ui.label("Lens size (meters)");
        ui.add(egui::Slider::new(
            &mut self.options.lens.size_meters,
            self.options.lens.size_meters_min..=self.options.lens.size_meters_max,
        ));
        ui.end_row();
        ui.label("Zoom speed")
            .on_hover_text("How fast the lens zooms in/out when scrolling.");
        ui.add(egui::Slider::new(
            &mut self.options.lens.scroll_speed_factor,
            0.0..=1.0,
        ));
    }
}
