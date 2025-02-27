use std::sync::{Arc, Mutex};

use eframe::egui;
use log::info;
use rfd::{AsyncFileDialog, FileHandle};

use crate::app::AppState;
use crate::map_pose::{self, MapPose};
use crate::wasm::async_data::AsyncData;

#[cfg(target_arch = "wasm32")]
async fn load_map_pose(data: &mut AsyncData, file_handle: FileHandle, map_name: String) {
    match MapPose::from_bytes(&file_handle.read().await) {
        Ok(map_pose) => {
            info!(
                "Loaded map pose file as bytes: {:?}",
                file_handle.file_name()
            );
            data.map_poses.insert(map_name, map_pose);
        }
        Err(e) => {
            data.error
                .clone_from(&format!("Error loading map pose file: {}", e.message));
            return;
        }
    }
}

#[cfg(target_arch = "wasm32")]
async fn save_map_pose(
    data: &mut AsyncData,
    file_handle: FileHandle,
    map_name: &str,
    map_pose: &MapPose,
) {
    if let Ok(bytes) = map_pose.to_bytes() {
        match file_handle.write(bytes.as_slice()).await {
            Ok(_) => {
                info!(
                    "Saved map pose file for map {} as bytes: {:?}",
                    map_name,
                    file_handle.file_name()
                );
            }
            Err(e) => {
                data.error.clone_from(&format!(
                    "Error saving map pose file for map {}: {:?}",
                    map_name, e
                ));
            }
        }
    } else {
        data.error.clone_from(&format!(
            "Error serializing map pose file for map {}",
            map_name
        ));
    }
}

/// Pick a map pose file via rfd dialog (websys -> <input> html).
#[cfg(target_arch = "wasm32")]
fn pick_load_map_pose(data: Arc<Mutex<AsyncData>>, map_name: String) {
    let dialog = AsyncFileDialog::new()
        .set_title("Select a map pose YAML file:")
        .add_filter("YAML", &["yml", "yaml"]);

    let future = dialog.pick_file();

    wasm_bindgen_futures::spawn_local(async move {
        let Ok(mut locked_data) = data.try_lock() else {
            return;
        };

        if let Some(file_handle) = future.await {
            load_map_pose(&mut locked_data, file_handle, map_name).await;
        }
    });
}

#[cfg(target_arch = "wasm32")]
fn pick_save_map_pose(data: Arc<Mutex<AsyncData>>, map_name: String, map_pose: MapPose) {
    let dialog = AsyncFileDialog::new()
        .set_title("Save map pose YAML file:")
        .add_filter("YAML", &["yml", "yaml"])
        .set_file_name("map_pose.yaml");

    let future = dialog.save_file();

    wasm_bindgen_futures::spawn_local(async move {
        let Ok(mut locked_data) = data.try_lock() else {
            return;
        };

        let Some(file_handle) = future.await else {
            return;
        };
        if let Ok(bytes) = map_pose.to_bytes() {
            match file_handle.write(bytes.as_slice()).await {
                Ok(_) => {
                    info!(
                        "Saved map pose file for map {} as bytes: {:?}",
                        map_name,
                        file_handle.file_name()
                    );
                }
                Err(e) => {
                    locked_data.error.clone_from(&format!(
                        "Error saving map pose file for map {}: {:?}",
                        map_name, e
                    ));
                }
            }
        } else {
            locked_data.error.clone_from(&format!(
                "Error serializing map pose file for map {}",
                map_name
            ));
        }
    });
}

impl AppState {
    /// wasm-compatible replacement for load_map_pose_button.
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn load_map_pose_button(&mut self, ui: &mut egui::Ui, map_name: &str) {
        if ui.button("📂 Load Map Pose").clicked() {
            pick_load_map_pose(self.data.wasm_io.clone(), map_name.to_string());
        }
        // ui repaint is needed to trigger the handler also without ui interaction.
        ui.ctx().request_repaint();
    }

    /// wasm-compatible replacement for save_map_pose_button.
    pub(crate) fn save_map_pose_button(&mut self, ui: &mut egui::Ui, map_name: &str) {
        if ui.button("💾 Save Map Pose").clicked() {
            let Some(map_pose) = self.data.maps.get(map_name).map(|map| map.pose.clone()) else {
                self.status.error = format!("Can't save pose, map {} not found.", map_name);
                return;
            };

            pick_save_map_pose(self.data.wasm_io.clone(), map_name.to_string(), map_pose);
        }
        // ui repaint is needed to trigger the handler also without ui interaction.
        ui.ctx().request_repaint();
    }
}
