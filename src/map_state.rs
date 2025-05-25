use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::option::Option;
use std::sync::Arc;

use eframe::egui;

use crate::image_pyramid::ImagePyramid;
use crate::map_pose::MapPose;
use crate::meta::Meta;
use crate::render_options::TextureFilter;
use crate::texture_state::TextureState;
use crate::value_interpretation::ValueInterpretation;

#[derive(Serialize, Deserialize)]
pub struct MapState {
    pub meta: Meta,
    pub pose: MapPose,
    pub visible: bool,
    pub tint: Option<egui::Color32>,
    pub color_to_alpha: Option<egui::Color32>,
    #[serde(default)]
    pub texture_filter: TextureFilter,
    #[serde(default)]
    pub use_value_interpretation: bool,

    // The image pyramid is an Arc to allow sharing it for multiple textures.
    #[serde(skip_serializing, skip_deserializing)]
    pub image_pyramid: Arc<ImagePyramid>,
    /// Texture states of the map, i.e. all active textures created from this map's image
    /// for different client IDs (keys of the HashMap).
    /// See also `get_or_create_texture_state()`.
    #[serde(skip_serializing, skip_deserializing)]
    pub texture_states: HashMap<String, TextureState>,
}

impl MapState {
    /// Returns a mutable reference to the texture state that belongs to `id`.
    /// Creates a new texture state if none exists yet for this ID.
    pub fn get_or_create_texture_state(&mut self, id: &str) -> &mut TextureState {
        self.texture_states
            .entry(id.to_string())
            .or_insert_with(|| TextureState::new(self.image_pyramid.clone()))
    }

    pub fn get_value_interpretation(&self) -> Option<&ValueInterpretation> {
        if self.use_value_interpretation {
            Some(&self.meta.value_interpretation)
        } else {
            None
        }
    }
}
