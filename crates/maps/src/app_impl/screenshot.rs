use eframe::egui;
use log::{debug, info};
use strum::Display;

use crate::app::{AppState, PendingScreenshot, ScreenshotResult};
use maps_rendering::image::from_egui_image;

#[cfg(target_arch = "wasm32")]
use crate::wasm::async_image_io;
#[cfg(not(target_arch = "wasm32"))]
use log::error;
#[cfg(not(target_arch = "wasm32"))]
use maps_io_ros::save_image;

#[derive(Clone, Copy, Debug, Display, PartialEq, Eq)]
pub enum Viewport {
    Full,
    Clipped,
}

impl AppState {
    pub(crate) fn request_screenshot(&mut self, ui: &egui::Ui, viewport: Viewport) {
        let mut pending_screenshot = self
            .pending_screenshot
            .lock()
            .expect("Screenshot callback mutex poisoned");
        if pending_screenshot.is_some() {
            return;
        }
        debug!("{viewport} screenshot requested for the next frame.");
        *pending_screenshot = Some(PendingScreenshot::Requested(viewport));
        drop(pending_screenshot);

        let ctx = ui.ctx().clone();
        // `request_screenshot` completes asynchronously, so its callback needs its own context handle.
        let repaint_ctx = ctx.clone();
        let pending_screenshot = self.pending_screenshot.clone();
        ctx.request_screenshot(move |image| {
            let mut pending_screenshot = pending_screenshot
                .lock()
                .expect("Screenshot callback mutex poisoned");
            if let Some(PendingScreenshot::Requested(viewport)) = pending_screenshot.take() {
                *pending_screenshot = Some(PendingScreenshot::Captured(ScreenshotResult {
                    image,
                    viewport,
                }));
            }
            // Ensure the frame with the pending screenshot is rendered immediately.
            repaint_ctx.request_repaint();
        });
    }

    pub(crate) fn screenshot_pending(&self) -> bool {
        self.pending_screenshot
            .lock()
            .expect("Screenshot callback mutex poisoned")
            .is_some()
    }

    pub(crate) fn clipped_screenshot_requested(&self) -> bool {
        self.pending_screenshot
            .lock()
            .expect("Screenshot callback mutex poisoned")
            .as_ref()
            .is_some_and(|screenshot| screenshot.viewport() == Viewport::Clipped)
    }

    pub(crate) fn handle_new_screenshot(&mut self, ctx: &egui::Context, clip_rect: &egui::Rect) {
        let mut pending_screenshot = self
            .pending_screenshot
            .lock()
            .expect("Screenshot callback mutex poisoned");
        let screenshot = pending_screenshot
            .take_if(|screenshot| screenshot.is_captured())
            .map(PendingScreenshot::into_captured);
        drop(pending_screenshot);

        if let Some(ScreenshotResult { image, viewport }) = screenshot {
            info!(
                "Captured {} screenshot.",
                viewport.to_string().to_lowercase()
            );

            let image = match viewport {
                Viewport::Full => from_egui_image(&image),
                Viewport::Clipped => from_egui_image(&image).crop(
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
