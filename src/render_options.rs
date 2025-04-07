use eframe::egui;
use serde::{Deserialize, Serialize};

/// Options for image rendering.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum TextureFilter {
    /// Linearly interpolate texels.
    /// Default option for smooth antialiased visualization.
    #[default]
    Smooth,
    /// Show texels as sharp squares.
    /// Useful when grid map image cells are investigated.
    Crisp,
}

impl TextureFilter {
    pub(crate) fn get(&self) -> egui::TextureOptions {
        match self {
            TextureFilter::Smooth => egui::TextureOptions::LINEAR,
            TextureFilter::Crisp => egui::TextureOptions::NEAREST,
        }
    }
}
