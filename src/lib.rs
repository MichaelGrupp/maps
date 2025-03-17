//! Library target of the `maps` app.
//!
//! The public components listed here can be reused in external applications.
//!
//! Note that public API is not the main focus of this app crate.
//! But feel free to use parts of it as you see fit.

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
mod texture_request;
mod texture_state;
mod tiles;
mod tiles_behavior;
pub mod value_colormap;
pub mod value_interpretation;

#[cfg(target_arch = "wasm32")]
mod wasm;
