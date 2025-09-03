use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use eframe::egui;
use log::info;
use rfd::{AsyncFileDialog, FileHandle};

use crate::app::AppState;
use crate::error::Error;
use crate::image::load_image_from_bytes;
use crate::image_pyramid::ImagePyramid;
use crate::meta::Meta;
use crate::wasm::async_data::AsyncData;

const YAML_EXTENSIONS: [&str; 2] = ["yml", "yaml"];
const IMAGE_EXTENSIONS: [&str; 4] = ["png", "jpg", "jpeg", "pgm"];
const ALL_EXTENSIONS: [&str; 6] = ["png", "jpg", "jpeg", "pgm", "yml", "yaml"];

#[cfg(target_arch = "wasm32")]
async fn load_image(file_handle: &FileHandle) -> Result<Arc<ImagePyramid>, Error> {
    let img_bytes = file_handle.read().await;
    let image = load_image_from_bytes(&img_bytes)?;
    info!(
        "Loaded image file from bytes: {:?}",
        file_handle.file_name()
    );
    Ok(Arc::new(ImagePyramid::new(image)))
}

#[cfg(target_arch = "wasm32")]
async fn load_meta(file_handle: &FileHandle) -> Result<Meta, Error> {
    Meta::load_from_bytes(
        file_handle.read().await.as_slice(),
        file_handle.file_name().as_str(),
    )
}

fn file_handles_with_extension<'a>(
    file_handles: &'a [FileHandle],
    extensions: &[&str],
) -> Vec<&'a FileHandle> {
    file_handles
        .iter()
        .filter(|file_handle| {
            matches!(
                PathBuf::from(file_handle.file_name())
                    .extension()
                    .unwrap_or_default()
                    .to_str()
                    .expect("non-utf8 extension?"),
                ext if extensions.contains(&ext)
            )
        })
        .collect()
}

/// Pick map YAML and image files via rfd dialog (websys -> <input> html).
#[cfg(target_arch = "wasm32")]
fn pick_map_files(data: Arc<Mutex<AsyncData>>) {
    let dialog = AsyncFileDialog::new()
        .set_title("Select pairs of YAML and corresponding image files:")
        .add_filter("YAML or image", &ALL_EXTENSIONS);

    let future = dialog.pick_files();

    wasm_bindgen_futures::spawn_local(async move {
        if data.try_lock().is_err() {
            return;
        }

        let Some(file_handles) = future.await else {
            return;
        };
        let yaml_handles = file_handles_with_extension(&file_handles, &YAML_EXTENSIONS);
        let image_handles = file_handles_with_extension(&file_handles, &IMAGE_EXTENSIONS);
        if yaml_handles.len() != image_handles.len() {
            if let Ok(mut locked_data) = data.try_lock() {
                locked_data
                    .error
                    .clone_from(&"Select a YAML and image file pair for each map.".to_string());
            }
            return;
        }

        for yaml_file in yaml_handles {
            let meta = match load_meta(yaml_file).await {
                Ok(meta) => meta,
                Err(e) => {
                    if let Ok(mut locked_data) = data.try_lock() {
                        locked_data.error = e.to_string();
                    }
                    return;
                }
            };

            let expected_image = &meta.image_path;
            match image_handles
                .iter()
                .find(|image_handle| PathBuf::from(image_handle.file_name()) == *expected_image)
            {
                Some(image_file) => match load_image(image_file).await {
                    Ok(image) => {
                        if let Ok(mut locked_data) = data.try_lock() {
                            locked_data.metas.push(meta);
                            locked_data.images.push(image);
                        }
                    }
                    Err(e) => {
                        if let Ok(mut locked_data) = data.try_lock() {
                            locked_data.error = e.to_string();
                        }
                        return;
                    }
                },
                None => {
                    if let Ok(mut locked_data) = data.try_lock() {
                        locked_data.error.clone_from(&format!(
                                "No matching image file found for {:?}. YAML metadata points to {:?}, but \
                                 this file was not selected. Make sure to select the correct image file \
                                 for each YAML file.",
                                yaml_file.file_name(),
                                expected_image
                            ));
                    }
                    return;
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
