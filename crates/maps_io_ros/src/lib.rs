//! `maps_io_ros` provides fundamental I/O for 2D ROS grid maps including:
//! metadata parsing, value interpretation, colormaps and map poses.
//!
//! See the [maps](https://crates.io/crates/maps) app crate for a full
//! GUI application that builds on top of this I/O library.
//!
//! This crate has minimal dependencies and can be used in
//! other robotics applications that work with ROS map files.

pub mod error;
pub mod image;
pub mod map_pose;
pub mod meta;
mod os_helpers;
pub mod value_colormap;
pub mod value_interpretation;

// Re-export commonly used types.
pub use error::{Error, Result};
pub use image::{load_image, load_image_from_bytes};
pub use map_pose::MapPose;
pub use meta::Meta;
pub use value_colormap::ColorMap;
pub use value_interpretation::ValueInterpretation;
