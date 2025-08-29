use eframe::egui;
use serde::{Deserialize, Serialize};

/// Textures above this size should be cropped.
pub const fn default_crop_threshold() -> u32 {
    6000
}

/// Options for image rendering.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum TextureFilter {
    /// Linearly interpolate texels.
    /// Default option for smooth antialiased visualization.
    Smooth,
    /// Show texels as sharp squares.
    /// Useful when grid map image cells are investigated.
    Crisp,
    /// Chooses the best option based on the number of pixels per texel:
    /// Crisp when magnified, Smooth when minified.
    #[default]
    Auto,
}

impl TextureFilter {
    pub(crate) fn get(&self, points_per_pixel: f32) -> egui::TextureOptions {
        match self {
            TextureFilter::Smooth => egui::TextureOptions::LINEAR,
            TextureFilter::Crisp => egui::TextureOptions::NEAREST,
            TextureFilter::Auto => {
                if points_per_pixel > 1.0 {
                    egui::TextureOptions::NEAREST
                } else {
                    egui::TextureOptions::LINEAR
                }
            }
        }
    }
}
