use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use eframe::egui;
use log::info;
use rfd::{AsyncFileDialog, FileHandle};

use crate::app::AppState;
use crate::image::load_image_from_bytes;
use crate::image_pyramid::ImagePyramid;
use crate::meta::Meta;
use crate::wasm::async_data::AsyncData;

#[cfg(target_arch = "wasm32")]
async fn load_image(data: &mut AsyncData, file_handle: FileHandle) {
    let img_bytes = file_handle.read().await;
    match load_image_from_bytes(&img_bytes) {
        Ok(image) => {
            info!(
                "Loaded image file from bytes: {:?}",
                file_handle.file_name()
            );
            data.images.push(Arc::new(ImagePyramid::new(image)));
        }
        Err(e) => {
            data.error
                .clone_from(&format!("Error loading image file: {:?}", e));
            return;
        }
    }
}

#[cfg(target_arch = "wasm32")]
async fn load_meta(data: &mut AsyncData, file_handle: FileHandle) {
    match Meta::load_from_bytes(
        file_handle.read().await.as_slice(),
        file_handle.file_name().as_str(),
    ) {
        Ok(meta) => {
            info!(
                "Loaded metadata file as bytes: {:?}",
                file_handle.file_name()
            );
            data.metas.push(meta);
        }
        Err(e) => {
            data.error
                .clone_from(&format!("Error loading metadata file: {}", e.message));
            return;
        }
    }
}

/// Pick map YAML and image files via rfd dialog (websys -> <input> html).
#[cfg(target_arch = "wasm32")]
fn pick_map_files(data: Arc<Mutex<AsyncData>>) {
    let dialog = AsyncFileDialog::new()
        .set_title("Select a map YAML and the corresponding image file:")
        .add_filter(
            "YAML or image",
            &["yml", "yaml", "png", "jpg", "jpeg", "pgm"],
        );

    let future = dialog.pick_files();

    wasm_bindgen_futures::spawn_local(async move {
        let Ok(mut locked_data) = data.try_lock() else {
            return;
        };

        if let Some(file_handles) = future.await {
            if file_handles.len() != 2 {
                locked_data
                    .error
                    .clone_from(&"Select exactly one YAML and one image file.".to_string());
                return;
            }
            for file_handle in file_handles {
                match PathBuf::from(file_handle.file_name())
                    .extension()
                    .unwrap_or_default()
                    .to_str()
                    .expect("failed to get extension")
                {
                    "yaml" | "yml" => {
                        load_meta(&mut locked_data, file_handle).await;
                    }
                    "png" | "jpg" | "jpeg" | "pgm" => {
                        load_image(&mut locked_data, file_handle).await;
                    }
                    _ => {
                        locked_data.error.clone_from(&format!(
                            "Unsupported file type: {:?}",
                            file_handle.file_name()
                        ));
                    }
                }
            }
        }
    });
}

impl AppState {
    /// wasm-compatible replacement for load_meta_button.
    /// Behaves differently because it needs to be async and requires to
    /// load both map and image due to missing filesystem access.
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn load_meta_button(&mut self, ui: &mut egui::Ui) {
        // Use rfd for wasm file dialog.
        if ui.button("📂 Load Maps").clicked() {
            pick_map_files(self.data.wasm_io.clone());
        }
        // ui repaint is needed to trigger the handler also without ui interaction.
        ui.ctx().request_repaint();
    }
}
