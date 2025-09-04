//! Library target of the `maps` app.
//!
//! The public components listed here can be reused in external applications.
//!
//! Note that public API is not the main focus of this app crate.
//! But feel free to use parts of it as you see fit.

// Enable selected pedantic lints.
// https://rust-lang.github.io/rust-clippy/master/index.html#uninlined_format_args
#![warn(clippy::uninlined_format_args)]

pub mod app;
mod app_impl;
mod draw_order;
pub mod error;
mod grid;
mod grid_options;
mod image;
mod image_pyramid;
mod lens;
pub mod map_pose;
mod map_state;
pub mod meta;
mod movable;
mod path_helpers;
pub mod persistence;
mod rect_helpers;
mod render_options;
mod texture_cache;
mod texture_request;
mod texture_state;
mod tiles;
mod tiles_behavior;
mod tracing;
pub mod value_colormap;
pub mod value_interpretation;

#[cfg(target_arch = "wasm32")]
mod wasm;
