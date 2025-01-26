use std::env::current_dir;
use std::path::PathBuf;
use std::sync::Arc;

use eframe::egui;
use egui_file_dialog::FileDialog;
use log::{debug, error, info};

use crate::image::load_image;
use crate::image_pyramid::ImagePyramid;
use crate::map_state::MapState;
use crate::meta::Meta;
use crate::persistence;
use crate::texture_state::TextureState;
use crate::tiles::Pane;

use crate::app::{AppState, Error};
use crate::map_pose::MapPose;

impl AppState {
    pub fn make_yaml_file_dialog(initial_dir: &Option<PathBuf>) -> FileDialog {
        FileDialog::new()
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0., 0.))
            .add_file_filter(
                "yaml",
                Arc::new(|path| {
                    ["yml", "yaml"].contains(
                        &path
                            .extension()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or_default(),
                    )
                }),
            )
            .default_file_filter("yaml")
            .initial_directory(
                initial_dir
                    .clone()
                    .unwrap_or(current_dir().expect("wtf no cwd??")),
            )
    }

    pub fn make_toml_file_dialog(initial_dir: &Option<PathBuf>) -> FileDialog {
        FileDialog::new()
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0., 0.))
            .add_file_filter(
                "toml",
                Arc::new(|path| {
                    ["toml"].contains(
                        &path
                            .extension()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or_default(),
                    )
                }),
            )
            .default_file_filter("toml")
            .initial_directory(
                initial_dir
                    .clone()
                    .unwrap_or(current_dir().expect("wtf no cwd??")),
            )
    }

    fn load_meta(&mut self, yaml_path: PathBuf) -> Result<bool, Error> {
        match Meta::load_from_file(yaml_path) {
            Ok(meta) => match self.load_map(meta) {
                Ok(_) => Ok(true),
                Err(e) => Err(e),
            },
            Err(e) => Err(Error {
                message: format!("Error loading metadata file: {}", e.message),
            }),
        }
    }

    pub fn load_meta_button(&mut self, ui: &mut egui::Ui) {
        if ui.button("📂 Load Maps").clicked() {
            self.load_meta_file_dialog.pick_multiple();
        }
        self.load_meta_file_dialog.update(ui.ctx());

        if let Some(paths) = self.load_meta_file_dialog.take_picked_multiple() {
            for path in paths {
                ui.ctx().request_repaint();
                match self.load_meta(path.clone()) {
                    Ok(_) => {
                        info!("Loaded metadata file: {:?}", path);
                        // Start from the same path the next time.
                        self.load_meta_file_dialog.config_mut().initial_directory = path;
                    }
                    Err(e) => {
                        self.status.error = e.message;
                        error!("{}", self.status.error);
                    }
                }
            }
        }
    }

    pub fn load_map(&mut self, meta: Meta) -> Result<(), Error> {
        match load_image(&meta.image_path) {
            Ok(image) => {
                self.tile_manager.add_pane(Pane {
                    id: meta.yaml_path.to_str().unwrap().to_owned(),
                });
                let image_pyramid = ImagePyramid::new(image);
                let name = meta.yaml_path.to_str().unwrap().to_owned();
                self.maps.insert(
                    name.clone(),
                    MapState {
                        meta,
                        pose: MapPose::default(),
                        visible: true,
                        texture_state: TextureState::new(image_pyramid),
                        overlay_texture: None,
                        tint: None,
                        color_to_alpha: None,
                    },
                );
                info!("Loaded map: {}", name);
                Ok(())
            }
            Err(e) => Err(Error {
                message: format!(
                    "Error loading image {:?}: {}",
                    &meta.image_path,
                    e.to_string()
                ),
            }),
        }
    }

    pub fn delete(&mut self, to_delete: &Vec<String>) {
        for name in to_delete {
            info!("Removing {}", name);
            self.maps.remove(name);
            self.tile_manager.remove_pane(name);
            if let Some(active_lens) = &self.options.active_lens {
                if active_lens == name {
                    self.options.active_lens = None;
                }
            }
            if let Some(active_tint_selection) = &self.options.tint_settings.active_tint_selection {
                if active_tint_selection == name {
                    self.options.tint_settings.active_tint_selection = None;
                }
            }
            if self.options.pose_edit.selected_map == *name {
                self.options.pose_edit.selected_map = "".to_string();
            }
        }
    }

    pub fn load_map_pose_button(&mut self, ui: &mut egui::Ui, map_name: &str) {
        if ui
            .button("📂 Load Pose")
            .on_hover_text("Load a map pose from a YAML file.")
            .clicked()
        {
            self.load_map_pose_file_dialog.pick_file();
        }
        self.load_map_pose_file_dialog.update(ui.ctx());

        if let Some(path) = self.load_map_pose_file_dialog.take_picked() {
            debug!("Loading pose file: {:?}", path);
            match MapPose::from_yaml_file(&path) {
                Ok(map_pose) => {
                    info!("Loaded pose file: {:?}", path);
                    self.maps.get_mut(map_name).unwrap().pose = map_pose;
                    // Start from the same path the next time, also for saving.
                    self.load_map_pose_file_dialog
                        .config_mut()
                        .initial_directory = path.clone();
                    self.save_map_pose_file_dialog
                        .config_mut()
                        .initial_directory = path;
                }
                Err(e) => {
                    self.status.error = format!("Error loading pose file: {}", e.message);
                    error!("{}", self.status.error);
                }
            }
        }
    }

    pub fn save_map_pose_button(&mut self, ui: &mut egui::Ui, map_name: &str) {
        if ui
            .button("💾 Save Pose")
            .on_hover_text("Save the map pose to a YAML file.")
            .clicked()
        {
            self.save_map_pose_file_dialog.save_file();
        }
        self.save_map_pose_file_dialog.update(ui.ctx());

        if let Some(path) = self.save_map_pose_file_dialog.take_picked() {
            ui.ctx().request_repaint();
            debug!("Saving pose file: {:?}", path);
            match self.maps.get(map_name).unwrap().pose.to_yaml_file(&path) {
                Ok(_) => {
                    info!("Saved pose file: {:?}", path);
                    // Start from the same path the next time, also for loading.
                    self.save_map_pose_file_dialog
                        .config_mut()
                        .initial_directory = path.clone();
                    self.load_map_pose_file_dialog
                        .config_mut()
                        .initial_directory = path;
                }
                Err(e) => {
                    self.status.error = format!("Error saving pose file: {}", e.message);
                    error!("{}", self.status.error);
                }
            }
        }
    }

    pub fn load_session_button(&mut self, ui: &mut egui::Ui) {
        if ui
            .button("📂 Load Session")
            .on_hover_text("Load a session from a file.")
            .clicked()
        {
            self.load_session_file_dialog.pick_file();
        }
        self.load_session_file_dialog.update(ui.ctx());

        if let Some(path) = self.load_session_file_dialog.take_picked() {
            debug!("Loading session file: {:?}", path);
            match persistence::load_map_states(&path) {
                Ok(deserialized_map_states) => {
                    info!("Loaded session file: {:?}", path);
                    // Start from the same path the next time.
                    self.load_session_file_dialog.config_mut().initial_directory = path.clone();
                    self.save_session_file_dialog.config_mut().initial_directory = path;
                    // Not everything gets serialized. Load actual data.
                    for (name, map) in deserialized_map_states {
                        debug!("Restoring map: {}", name);
                        match self.load_map(map.meta) {
                            Ok(_) => {
                                let map_state = self.maps.get_mut(&name).unwrap();
                                map_state.pose = map.pose;
                                map_state.visible = map.visible;
                                map_state.tint = map.tint;
                                map_state.color_to_alpha = map.color_to_alpha;
                            }
                            Err(e) => {
                                self.status.error = e.message;
                                error!("{}", self.status.error);
                            }
                        }
                    }
                }
                Err(e) => {
                    self.status.error = format!("Error loading session file: {}", e.message);
                    error!("{}", self.status.error);
                }
            }
        }
    }

    pub fn save_session_button(&mut self, ui: &mut egui::Ui) {
        if ui
            .button("💾 Save Session")
            .on_hover_text("Save the current session to a file.")
            .clicked()
        {
            self.save_session_file_dialog.save_file();
        }
        self.save_session_file_dialog.update(ui.ctx());

        if let Some(path) = self.save_session_file_dialog.take_picked() {
            debug!("Saving session file: {:?}", path);
            match persistence::save_map_states(&path, &self.maps) {
                Ok(_) => {
                    info!("Saved session file: {:?}", path);
                    // Start from the same path the next time.
                    self.save_session_file_dialog.config_mut().initial_directory = path.clone();
                    self.load_session_file_dialog.config_mut().initial_directory = path;
                }
                Err(e) => {
                    self.status.error = format!("Error saving session file: {}", e.message);
                    error!("{}", self.status.error);
                }
            }
        }
    }
}
