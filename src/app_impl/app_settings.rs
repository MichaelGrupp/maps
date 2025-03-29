use eframe::egui;

use crate::app::AppState;
use crate::app_impl::ui_helpers::section_heading;

impl AppState {
    pub(crate) fn app_settings(&mut self, ui: &mut egui::Ui) {
        if !section_heading(ui, "App", &mut self.options.collapsed.app_settings) {
            return;
        }
        ui.end_row();
        ui.label("Autosave options").on_hover_text(
            "Save the app options when the window is closed.\n\
            The options are loaded when the app is started.",
        );

        #[cfg(not(target_arch = "wasm32"))]
        ui.checkbox(&mut self.options.persistence.autosave, "");

        #[cfg(target_arch = "wasm32")]
        ui.label(
            egui::RichText::new("Only supported in native builds.")
                .weak()
                .italics(),
        );
    }
}
