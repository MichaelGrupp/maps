use eframe::egui;
use log::{debug, error, info};

use crate::app::AppState;
use crate::image::from_egui_image;

#[derive(Clone, Debug)]
pub enum Viewport {
    Full,
    Clipped,
}

impl AppState {
    pub(crate) fn request_screenshot(&self, ui: &egui::Ui, viewport: Viewport) {
        ui.ctx()
            .send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                viewport,
            )));
        debug!("Screenshot requested for the next frame.");
    }

    pub(crate) fn handle_new_screenshot(&mut self, ctx: &egui::Context, clip_rect: &egui::Rect) {
        let event_data = ctx.input(|i| {
            i.events
                .iter()
                .filter_map(|e| {
                    if let egui::Event::Screenshot {
                        image, user_data, ..
                    } = e
                    {
                        Some((image.clone(), user_data.clone()))
                    } else {
                        None
                    }
                })
                .last()
        });

        if let Some(event) = event_data {
            info!("Captured screenshot.");
            self.save_screenshot_dialog.save_file();

            let viewport: Viewport = match event.1.data {
                Some(data) => data
                    .downcast_ref::<Viewport>()
                    .expect("Failed to downcast viewport data.")
                    .clone(),
                _ => {
                    error!("Invalid viewport data for screenshot.");
                    return;
                }
            };
            let image = match viewport {
                Viewport::Full => from_egui_image(&event.0),
                Viewport::Clipped => from_egui_image(&event.0).crop(
                    clip_rect.min.x as u32,
                    clip_rect.min.y as u32,
                    clip_rect.width() as u32,
                    clip_rect.height() as u32,
                ),
            };
            self.data.screenshot = Some(image);
        }
        self.save_screenshot_dialog.update(ctx);

        if let Some(file_path) = self.save_screenshot_dialog.take_picked() {
            if let Some(image) = self.data.screenshot.take() {
                match image.save(file_path.clone()) {
                    Ok(_) => {
                        info!("Saved screenshot to {:?}", file_path);
                    }
                    Err(e) => {
                        self.status.error = format!("Error saving screenshot: {:?}", e.to_string());
                        error!("{}", self.status.error);
                    }
                }
            }
        }
    }
}
