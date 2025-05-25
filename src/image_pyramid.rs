use std::cmp::max;
use std::collections::HashMap;

use eframe::egui;
use log::{debug, trace};

use crate::image::{fit_image, to_rgba8};

// Side lengths used for the image pyramid levels.
// These shall correspond roughly to zoom levels w.r.t. original images.
const SIZES: [u32; 5] = [8000, 4000, 2000, 1000, 500];

// Stores downscaled versions of an image for discrete sizes.
// Intended for efficient on-screen rendering of images at different zoom levels.
#[derive(Default)]
pub struct ImagePyramid {
    pub original: image::DynamicImage,
    levels_by_size: HashMap<u32, image::DynamicImage>,
    aspect_ratio: f32,
    pub original_size: egui::Vec2,
    pub original_has_alpha: bool,
}

impl ImagePyramid {
    pub fn new(original: image::DynamicImage) -> ImagePyramid {
        // Always add an alpha channel, if not present, to support our image operations.
        // DynamicImage allows conversions, but we do it once here for performance reasons.
        // Use always RGBA8 internally.
        let original_has_alpha = original.color().has_alpha();
        let original = to_rgba8(original);

        let original_size = egui::Vec2::new(original.width() as f32, original.height() as f32);
        ImagePyramid {
            levels_by_size: {
                let mut levels: HashMap<u32, image::DynamicImage> = HashMap::new();
                let mut parent = None;
                for size in SIZES {
                    let image_to_downscale = match levels.get(&parent.unwrap_or(0)) {
                        Some(parent_level) => parent_level,
                        None => &original,
                    };
                    if max(original.width(), original.height()) < size {
                        continue;
                    }
                    debug!(
                        "Creating pyramid level for target size {} from {} image size {:?}",
                        size,
                        parent.map_or("original".to_string(), |_| "parent".to_string()),
                        (image_to_downscale.width(), image_to_downscale.height())
                    );
                    let level = fit_image(
                        image_to_downscale,
                        egui::Vec2::new(size as f32, size as f32),
                    );
                    levels.insert(size, level);
                    parent = Some(size);
                }
                levels
            },
            original,
            aspect_ratio: original_size.x / original_size.y,
            original_size,
            original_has_alpha,
        }
    }

    pub fn get_level(&self, size: egui::Vec2) -> &image::DynamicImage {
        // Get the closest size that is larger or equal to the requested size,
        // considering the aspect ratio of the original image for the dimension.
        let scale = (size.x / self.original_size.x).min(size.y / self.original_size.y);
        let dim = if self.aspect_ratio >= 1. {
            scale * self.original_size.x
        } else {
            scale * self.original_size.y
        };
        match SIZES
            .iter()
            .rev()
            .find(|&&s| s >= dim as u32 && self.levels_by_size.contains_key(&s))
        {
            Some(closest) => {
                trace!("Returning pyramid level for size: {}", closest);
                self.levels_by_size
                    .get(closest)
                    .expect("non-existing pyramid level")
            }
            None => {
                trace!(
                    "No pyramid level larger or equal {} found, returning original.",
                    size
                );
                &self.original
            }
        }
    }

    pub fn num_levels(&self) -> usize {
        self.levels_by_size.len()
    }
}
