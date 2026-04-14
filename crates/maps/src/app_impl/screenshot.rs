use eframe::egui;
use log::{debug, error, info};
use strum::Display;

use crate::app::AppState;
use maps_rendering::image::from_egui_image;

#[cfg(target_arch = "wasm32")]
use crate::wasm::async_image_io;
#[cfg(not(target_arch = "wasm32"))]
use maps_io_ros::save_image;

#[derive(Clone, Debug, Display)]
pub enum Viewport {
    Full,
    Clipped,
}

impl AppState {
    pub(crate) fn request_screenshot(&self, ui: &egui::Ui, viewport: Viewport) {
        debug!("{viewport} screenshot requested for the next frame.");
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
                let mut dialog = rfd::FileDialog::new()
                    .add_filter("PNG", &["png"])
                    .set_file_name("maps_screenshot.png");
                if let Some(dir) = &self.last_file_dir {
                    dialog = dialog.set_directory(dir);
                }
                if let Some(file_path) = dialog.save_file() {
                    match save_image(&file_path, &image) {
                        Ok(_) => {
                            info!("Saved screenshot to {file_path:?}");
                            self.last_file_dir =
                                file_path.parent().map(std::path::Path::to_path_buf);
                        }
                        Err(e) => {
                            self.status.error = format!("Failed to save screenshot: {e}");
                            error!("{e}");
                        }
                    }
                }
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
    }
}
