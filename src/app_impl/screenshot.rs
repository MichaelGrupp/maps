use eframe::egui;
use log::{debug, error, info};
use strum::Display;

use crate::app::AppState;
use crate::image::from_egui_image;

#[cfg(target_arch = "wasm32")]
use crate::wasm::async_image_io;

#[derive(Clone, Debug, Display)]
pub enum Viewport {
    Full,
    Clipped,
}

impl AppState {
    pub(crate) fn request_screenshot(&self, ui: &egui::Ui, viewport: Viewport) {
        debug!(
            "{} screenshot requested for the next frame.",
            viewport.to_string()
        );
        ui.ctx()
            .send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                viewport,
            )));
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
                .next_back()
        });

        if let Some(event) = event_data {
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
            info!(
                "Captured {} screenshot.",
                viewport.to_string().to_lowercase()
            );

            let image = match viewport {
                Viewport::Full => from_egui_image(&event.0),
                Viewport::Clipped => from_egui_image(&event.0).crop(
                    (clip_rect.min.x * ctx.pixels_per_point()) as u32,
                    (clip_rect.min.y * ctx.pixels_per_point()) as u32,
                    (clip_rect.width() * ctx.pixels_per_point()) as u32,
                    (clip_rect.height() * ctx.pixels_per_point()) as u32,
                ),
            };

            #[cfg(not(target_arch = "wasm32"))]
            {
                self.data.screenshot = Some(image);
                self.save_screenshot_dialog.save_file();
            }
            #[cfg(target_arch = "wasm32")]
            {
                async_image_io::pick_save_png(
                    self.data.wasm_io.clone(),
                    "maps_screenshot.png".to_string(),
                    image,
                );
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.save_screenshot_dialog.update(ctx);

            if let Some(file_path) = self.save_screenshot_dialog.take_picked() {
                if let Some(image) = self.data.screenshot.take() {
                    match image.save(file_path.clone()) {
                        Ok(_) => {
                            info!("Saved screenshot to {:?}", file_path);
                        }
                        Err(e) => {
                            self.status.error =
                                format!("Error saving screenshot: {:?}", e.to_string());
                            error!("{}", self.status.error);
                        }
                    }
                }
            }
        }
    }
}
