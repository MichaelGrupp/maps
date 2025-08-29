use std::collections::HashMap;

use eframe::egui;

use crate::texture_request::TextureRequest;
use crate::value_interpretation::ValueInterpretation;

/// A cached texture handle with its appearance properties.
#[derive(Clone)]
struct CachedTexture {
    texture_handle: egui::TextureHandle,
    color_to_alpha: Option<egui::Color32>,
    thresholding: Option<ValueInterpretation>,
    texture_options: egui::TextureOptions,
}

impl CachedTexture {
    /// Returns true if this cached texture matches the appearance requirements.
    fn matches_appearance(&self, request: &TextureRequest) -> bool {
        self.color_to_alpha == request.color_to_alpha
            && self.thresholding == request.thresholding
            && self.texture_options == request.texture_options.unwrap_or_default()
    }
}

/// Cache for texture handles to avoid reloading textures when switching resolution levels,
/// unless some appearance properties change and a new texture needs to be created.
#[derive(Default)]
pub struct TextureCache {
    cache: HashMap<String, CachedTexture>,
}

impl TextureCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    fn generate_key(client: &str, pyramid_level: u32) -> String {
        format!("{}_{}", client, pyramid_level)
    }

    /// Queries the cache for a texture that matches the client, level, and appearance requirements.
    /// Returns None on cache-miss.
    pub fn query(
        &mut self,
        client: &str,
        pyramid_level: u32,
        request: &TextureRequest,
    ) -> Option<egui::TextureHandle> {
        let cache_key = Self::generate_key(client, pyramid_level);

        if let Some(cached_texture) = self.cache.get(&cache_key) {
            if cached_texture.matches_appearance(request) {
                return Some(cached_texture.texture_handle.clone());
            } else {
                // Remove outdated cache entry.
                self.cache.remove(&cache_key);
            }
        }

        None
    }

    pub fn store(
        &mut self,
        client: &str,
        pyramid_level: u32,
        texture_handle: egui::TextureHandle,
        request: &TextureRequest,
    ) {
        let cache_key = Self::generate_key(client, pyramid_level);
        let cached_texture = CachedTexture {
            texture_handle,
            color_to_alpha: request.color_to_alpha,
            thresholding: request.thresholding,
            texture_options: request.texture_options.unwrap_or_default(),
        };
        self.cache.insert(cache_key, cached_texture);
    }
}
