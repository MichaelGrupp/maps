//! Image rendering and texture management tailored to the [maps](https://crates.io/crates/maps) crate.
//!
//! For fast image editing previews in egui, with arbitrary scale & pose,
//! and potentially very large source images.

pub mod image;
pub mod image_pyramid;
mod rect_helpers;
pub mod render_options;
mod texture_cache;
pub mod texture_request;
pub mod texture_state;

// Re-export commonly used structs and types.
pub use image_pyramid::ImagePyramid;
pub use render_options::TextureFilter;
pub use texture_request::{ImagePlacement, NO_TINT, RotatedCropRequest, TextureRequest};
pub use texture_state::TextureState;
