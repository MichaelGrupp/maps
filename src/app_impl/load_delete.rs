use std::path::PathBuf;
use std::sync::Arc;

use eframe::egui;
use egui_file_dialog::FileDialog;
use log::debug;

use crate::image::load_image;
use crate::image_pyramid::ImagePyramid;
use crate::map_state::MapState;
use crate::meta::Meta;
use crate::texture_state::TextureState;
use crate::tiles::Pane;

use crate::app::{AppState, Error};
use crate::map_pose::MapPose;

impl AppState {
    pub fn make_yaml_file_dialog() -> FileDialog {
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
    }

    fn load_meta(&mut self, yaml_path: PathBuf) -> Result<bool, Error> {
        match Meta::load_from_file(yaml_path) {
            Ok(meta) => match self.load_image(meta) {
                Ok(_) => Ok(true),
                Err(e) => Err(e),
            },
            Err(e) => Err(Error {
                message: format!("Error loading metadata file: {:?}", e),
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
                        self.status_message = format!("Loaded metadata file: {:?}", path);
                    }
                    Err(e) => {
                        self.status_message =
                            format!("Error loading metadata file: {:?}", e.message);
                    }
                }
            }
        }
    }

    pub fn load_image(&mut self, meta: Meta) -> Result<(), Error> {
        self.status_message = format!("Loading image: {:?}", meta.image_path);
        match load_image(&meta.image_path) {
            Ok(image) => {
                self.tile_manager.add_pane(Pane {
                    id: meta.yaml_path.to_str().unwrap().to_owned(),
                });
                let image_pyramid = ImagePyramid::new(image);
                self.maps.insert(
                    meta.yaml_path.to_str().unwrap().to_owned(),
                    MapState {
                        meta,
                        pose: MapPose::default(),
                        visible: true,
                        texture_state: TextureState::new(image_pyramid),
                        overlay_texture: None,
                        tint: None,
                    },
                );
                Ok(())
            }
            Err(e) => Err(Error {
                message: format!("Error loading image: {:?}", e),
            }),
        }
    }

    pub fn delete(&mut self, to_delete: &Vec<String>) {
        for name in to_delete {
            self.maps.remove(name);
            self.tile_manager.remove_pane(&name);
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
        }
    }

    pub fn load_map_pose_button(&mut self, ui: &mut egui::Ui, map_name: &str) {
        if ui.button("📂 Load Pose").clicked() {
            self.load_map_pose_file_dialog.pick_file();
        }
        self.load_map_pose_file_dialog.update(ui.ctx());

        match self.load_map_pose_file_dialog.take_picked() {
            Some(path) => {
                debug!("Loading pose file: {:?}", path);
                match MapPose::from_yaml_file(&path) {
                    Ok(map_pose) => {
                        self.status_message = format!("Loaded pose file: {:?}", path);
                        self.maps.get_mut(map_name).unwrap().pose = map_pose;
                    }
                    Err(e) => {
                        self.status_message = format!("Error loading pose file: {:?}", e.message);
                    }
                }
            }
            None => (), // Nothing selected.
        }
    }

    pub fn save_map_pose_button(&mut self, ui: &mut egui::Ui, map_name: &str) {
        if ui.button("💾 Save Pose").clicked() {
            self.save_map_pose_file_dialog.save_file();
        }
        self.save_map_pose_file_dialog.update(ui.ctx());

        if let Some(path) = self.save_map_pose_file_dialog.take_picked() {
            ui.ctx().request_repaint();
            debug!("Saving pose file: {:?}", path);
            match self.maps.get(map_name).unwrap().pose.to_yaml_file(&path) {
                Ok(_) => {
                    self.status_message = format!("Saved pose file: {:?}", path);
                }
                Err(e) => {
                    self.status_message = format!("Error saving pose file: {:?}", e.message);
                }
            }
        }
    }
}
