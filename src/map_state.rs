use std::option::Option;

use eframe::egui;

use crate::map_pose::MapPose;
use crate::meta::Meta;
use crate::texture_state::TextureState;

pub struct MapState {
    pub meta: Meta,
    pub pose: MapPose,
    pub visible: bool,
    pub texture_state: TextureState,
    pub overlay_texture: Option<egui::TextureHandle>,
    pub tint: Option<egui::Color32>,
}
