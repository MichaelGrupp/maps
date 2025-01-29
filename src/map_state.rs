use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::option::Option;
use std::sync::Arc;

use eframe::egui;

use crate::image_pyramid::ImagePyramid;
use crate::map_pose::MapPose;
use crate::meta::Meta;
use crate::texture_state::TextureState;

#[derive(Serialize, Deserialize)]
pub struct MapState {
    pub meta: Meta,
    pub pose: MapPose,
    pub visible: bool,
    pub tint: Option<egui::Color32>,
    pub color_to_alpha: Option<egui::Color32>,

    // The image pyramid is an Arc to allow sharing it for multiple textures.
    #[serde(skip_serializing, skip_deserializing)]
    pub image_pyramid: Arc<ImagePyramid>,
    #[serde(skip_serializing, skip_deserializing)]
    pub texture_states: HashMap<String, TextureState>,
}

impl MapState {
    pub fn get_or_create_texture_state(&mut self, id: &str) -> &mut TextureState {
        self.texture_states
            .entry(id.to_string())
            .or_insert_with(|| TextureState::new(self.image_pyramid.clone()))
    }
}
