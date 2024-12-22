use std::option::Option;

use eframe::egui;

#[derive(Default)]
pub struct TextureState {
    pub image_response: Option<egui::Response>,
    pub texture_handle: Option<egui::TextureHandle>,
}
