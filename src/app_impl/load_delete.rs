use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use eframe::egui;
use log::{debug, error, info};

use crate::image::load_image;
use crate::image_pyramid::ImagePyramid;
use crate::map_state::MapState;
use crate::meta::Meta;
use crate::persistence;
use crate::render_options::TextureFilter;
use crate::tiles::Pane;

use crate::app::{AppState, Error};
use crate::app_impl::compat::migrate_old_egui_color;
use crate::map_pose::MapPose;
use crate::value_interpretation;

impl AppState {
    #[cfg(not(target_arch = "wasm32"))]
    fn load_meta(&mut self, yaml_path: &std::path::Path) -> Result<bool, Error> {
        let meta = Meta::load_from_file(yaml_path)?;
        self.load_map(meta)?;
        Ok(true)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn load_meta_button(&mut self, ui: &mut egui::Ui) {
        if ui.button("📂 Load Maps").clicked() {
            self.load_meta_file_dialog.pick_multiple();
        }
        self.load_meta_file_dialog.update(ui.ctx());

        if let Some(paths) = self.load_meta_file_dialog.take_picked_multiple() {
            for path in paths {
                ui.ctx().request_repaint();
                match self.load_meta(&path) {
                    Ok(_) => {
                        // Start from the same path the next time.
                        self.load_meta_file_dialog.config_mut().initial_directory = path;
                    }
                    Err(e) => {
                        self.status.error = e.to_string();
                        error!("{}", e);
                    }
                }
            }
        }
    }

    pub(crate) fn add_map(&mut self, name: &String, meta: Meta, image_pyramid: &Arc<ImagePyramid>) {
        let use_interpretation = meta.value_interpretation.explicit_mode;
        if use_interpretation {
            // This map has an explicitly specified value interpretation.
            // We need to set this to not loose the values in the next frame.
            self.options.tint_settings.active_tint_selection = Some(name.clone());
        }
        self.tile_manager.add_pane(Pane { id: name.clone() });
        self.data.maps.insert(
            name.clone(),
            MapState {
                meta,
                pose: MapPose::default(),
                visible: true,
                image_pyramid: image_pyramid.clone(),
                texture_states: HashMap::new(),
                tint: None,
                color_to_alpha: None,
                texture_filter: TextureFilter::default(),
                use_value_interpretation: use_interpretation,
            },
        );
        self.data.draw_order.add(name.clone());
        info!("Loaded map: {}", name);
        self.status.unsaved_changes = true;
    }

    pub(crate) fn load_map(&mut self, meta: Meta) -> Result<String, Error> {
        if !meta.image_path.exists() {
            return Err(Error::app(format!(
                "Image file doesn't exist: {:?}",
                meta.image_path
            )));
        }

        let image = if self.options.advanced.dry_run {
            info!("Dry-run mode, not loading image {:?}.", meta.image_path);
            Ok(image::DynamicImage::new_rgba8(0, 0))
        } else {
            load_image(&meta.image_path)
        }?;

        let image_pyramid = Arc::new(ImagePyramid::new(image));
        let name = meta
            .yaml_path
            .to_str()
            .expect("invalid unicode path, can't use as map name")
            .to_owned();
        self.add_map(&name, meta, &image_pyramid);
        Ok(name)
    }

    pub(crate) fn delete(&mut self, to_delete: &Vec<String>) {
        for name in to_delete {
            info!("Removing {}", name);
            self.data.maps.remove(name);
            self.data.draw_order.remove(name);
            self.tile_manager.remove_pane(name);
            if let Some(active_tool) = &self.status.active_tool
                && active_tool == name
            {
                self.status.active_tool = None;
            }
            if let Some(active_tint_selection) = &self.options.tint_settings.active_tint_selection
                && active_tint_selection == name
            {
                // Set the selection to one of the remaining maps if possible.
                // This avoids falling back to "All" (None) when there are still
                // other maps with potentially custom tints.
                self.options.tint_settings.active_tint_selection =
                    self.data.maps.keys().last().map(|s| s.to_string());
            }
            if self.options.pose_edit.selected_map == *name {
                self.options.pose_edit.selected_map = "".to_string();
            }
            self.status.unsaved_changes = true;
        }
    }

    pub(crate) fn add_map_pose(&mut self, map_name: &str, map_pose: MapPose) {
        if let Some(map) = self.data.maps.get_mut(map_name) {
            map.pose = map_pose;
            info!("Loaded pose for: {}", map_name);
            self.status.unsaved_changes = true;
        } else {
            error!("Tried to add pose to non-existing map: {}", map_name);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn load_map_pose_button(&mut self, ui: &mut egui::Ui, map_name: &str) {
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
                    self.add_map_pose(map_name, map_pose);
                    // Start from the same path the next time, also for saving.
                    self.load_map_pose_file_dialog
                        .config_mut()
                        .initial_directory = path.clone();
                    self.save_map_pose_file_dialog
                        .config_mut()
                        .initial_directory = path;
                    self.status.unsaved_changes = true;
                }
                Err(e) => {
                    self.status.error = e.to_string();
                    error!("{}", e);
                }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn save_map_pose_button(&mut self, ui: &mut egui::Ui, map_name: &str) {
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
            let Some(map) = self.data.maps.get(map_name) else {
                self.status.error = format!("Can't save pose, map doesn't exist: {map_name}");
                error!("{}", self.status.error);
                return;
            };
            match map.pose.to_yaml_file(&path) {
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
                    self.status.error = e.to_string();
                    error!("{}", e);
                }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_layout(&mut self, path: &PathBuf) -> Result<(), Error> {
        let lif_json = std::fs::read_to_string(path).map_err(|e| {
            Error::app(format!("Failed to read layout file {path:?}: {e}"))
        })?;
        let lif_file = crate::graph::vda_lif::LifFile::from_json(&lif_json)?;
        if lif_file.layouts.is_empty() {
            return Err(Error::app("No layouts found in LIF file."));
        }
        
        for lif_layout in &lif_file.layouts {
            let layout = crate::graph::layout::Layout::from_vda_lif(lif_layout);
            self.data.layouts.push(layout);
        }
        info!("Loaded {} layout(s) from {:?}", lif_file.layouts.len(), path);
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_layout_button(&mut self, ui: &mut egui::Ui) {
        if ui
            .button("📂 Load Layout")
            .on_hover_text("Load a layout from a VDA LIF file.")
            .on_disabled_hover_text("Only supported in native builds.")
            .clicked()
        {
            self.load_layout_file_dialog.pick_file();
        }
        self.load_layout_file_dialog.update(ui.ctx());
       
        if let Some(path) = self.load_layout_file_dialog.take_picked() {
            match self.load_layout(&path) {
                Ok(_) => {
                    // Start from the same path the next time.
                    self.load_layout_file_dialog
                        .config_mut()
                        .initial_directory = path;
                }
                Err(e) => {
                    self.status.error = e.to_string();
                    error!("{}", e);
                }
            }
        }
    }

    pub fn load_session(&mut self, path: &PathBuf) -> Result<(), Error> {
        let deserialized_session = persistence::load_session(path)?;

        // Start from the same path the next time.
        self.load_session_file_dialog.config_mut().initial_directory = path.clone();
        self.save_session_file_dialog.config_mut().initial_directory = path.clone();

        // Keep the draw order of the session, if it was saved.
        // If it was not saved (older versions), add_map() will take care of it.
        self.data
            .draw_order
            .extend(&deserialized_session.draw_order);

        // If the session has no version field, it was saved with maps < 1.7.0.
        // This means that the tint color was serialized with egui < 0.32 and
        // might need migration.
        let migrate_colors = deserialized_session.version.is_none();
        if migrate_colors {
            debug!("Session was saved with maps < 1.7.0, migrating serialized colors.");
        }

        // Not everything gets serialized. Load actual data.
        for (name, map) in deserialized_session.maps {
            debug!("Restoring map state: {}", name);
            let map_name = self.load_map(map.meta).inspect_err(|_| {
                // Make sure we have no dangling names in draw_order if we fail to load one map.
                self.data
                    .draw_order
                    .retain(|key| self.data.maps.contains_key(key));
            })?;
            let map_state = self.data.maps.get_mut(&map_name).expect("missing map");
            map_state.pose = map.pose;
            map_state.visible = map.visible;
            map_state.tint = map.tint;
            if migrate_colors {
                map_state.tint = migrate_old_egui_color(map_state.tint);
            }
            if map_state.tint.is_some()
                || map_state.meta.value_interpretation.mode != value_interpretation::Mode::Raw
            {
                // We need to set this because we would lose this map's tint
                // in the next frame if "All" is selected in the settings panel.
                self.options.tint_settings.active_tint_selection = Some(name.clone());
            }
            self.tile_manager
                .set_visible(map_name.as_str(), map.visible);
            map_state.color_to_alpha = map.color_to_alpha;
            if migrate_colors {
                map_state.color_to_alpha = migrate_old_egui_color(map_state.color_to_alpha);
            }
            self.status.unsaved_changes = false;
        }

        for (id, lens_pos) in deserialized_session.grid_lenses {
            debug!("Restoring lens {}", id);
            self.data.grid_lenses.insert(id, lens_pos);
        }

        Ok(())
    }

    pub(crate) fn load_session_button(&mut self, ui: &mut egui::Ui) {
        if ui
            .button("📂 Load Session")
            .on_hover_text("Load a session from a file.")
            .on_disabled_hover_text("Only supported in native builds.")
            .clicked()
        {
            self.load_session_file_dialog.pick_file();
        }
        self.load_session_file_dialog.update(ui.ctx());

        if let Some(path) = self.load_session_file_dialog.take_picked() {
            self.load_session(&path).unwrap_or_else(|e| {
                self.status.error = e.to_string();
                error!("{}", e);
            });
        }
    }

    pub(crate) fn save_session_button(&mut self, ui: &mut egui::Ui, quit_after_save: bool) {
        let text = if quit_after_save {
            "💾 Save Session and Quit"
        } else {
            "💾 Save Session"
        };
        if ui
            .button(text.to_owned())
            .on_hover_text("Save the current session to a file.")
            .on_disabled_hover_text("Only supported in native builds.")
            .clicked()
        {
            self.save_session_file_dialog.save_file();
            self.status.quit_after_save = quit_after_save;
            self.status.quit_modal_active = false;
            // TODO: Why is the dialog not visible when no button is visible?
            // Make sure the menu is visible from the menu panel.
            self.options.menu_visible = true;
        }
        self.save_session_file_dialog.update(ui.ctx());

        if let Some(path) = self.save_session_file_dialog.take_picked() {
            match persistence::save_session(&path, &self.data) {
                Ok(_) => {
                    // Start from the same path the next time.
                    self.save_session_file_dialog.config_mut().initial_directory = path.clone();
                    self.load_session_file_dialog.config_mut().initial_directory = path;
                    self.status.unsaved_changes = false;
                    if self.status.quit_after_save {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
                Err(e) => {
                    self.status.error = e.to_string();
                    error!("{}", e);
                }
            }
        }
    }
}
