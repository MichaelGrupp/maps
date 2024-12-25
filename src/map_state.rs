use std::option::Option;

use eframe::egui;

use crate::meta::Meta;
use crate::texture_state::TextureState;

pub struct MapState {
    pub meta: Meta,
    pub visible: bool,
    pub texture_state: TextureState,
    pub overlay_texture: Option<egui::TextureHandle>,
}
