use serde::{Deserialize, Serialize};
use std::option::Option;

use eframe::egui;

use crate::map_pose::MapPose;
use crate::meta::Meta;
use crate::texture_state::TextureState;

#[derive(Serialize, Deserialize)]
pub struct MapState {
    pub meta: Meta,
    pub pose: MapPose,
    pub visible: bool,
    #[serde(skip_serializing, skip_deserializing)]
    pub texture_state: TextureState,
    #[serde(skip_serializing, skip_deserializing)]
    pub overlay_texture: Option<egui::TextureHandle>,
    pub tint: Option<egui::Color32>,
    pub color_to_alpha: Option<egui::Color32>,
}
