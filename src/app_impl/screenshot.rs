use eframe::egui;
use log::{debug, error, info};

use crate::app::AppState;
use crate::image::from_egui_image;

impl AppState {
    pub(crate) fn request_screenshot(&self, ui: &egui::Ui) {
        ui.ctx()
            .send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
        debug!("Screenshot requested for the next frame.");
    }

    pub(crate) fn handle_new_screenshot(&mut self, ctx: &egui::Context) {
        let image = ctx.input(|i| {
            i.events
                .iter()
                .filter_map(|e| {
                    if let egui::Event::Screenshot { image, .. } = e {
                        Some(image.clone())
                    } else {
                        None
                    }
                })
                .last()
        });

        if let Some(image) = image {
            info!("Captured screenshot.");
            self.save_screenshot_dialog.save_file();
            self.data.screenshot = Some(image.clone());
        }
        self.save_screenshot_dialog.update(ctx);

        if let Some(file_path) = self.save_screenshot_dialog.take_picked() {
            if let Some(image) = self.data.screenshot.take() {
                match from_egui_image(&image).save(file_path.clone()) {
                    Ok(_) => {
                        info!("Saved screenshot to {:?}", file_path);
                    }
                    Err(e) => {
                        self.status.error = format!("Error saving screenshot: {:?}", e);
                        error!("{}", self.status.error);
                    }
                }
            }
        }
    }
}
