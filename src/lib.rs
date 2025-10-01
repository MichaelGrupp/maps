//! Library target of the `maps` app.
//!
//! The public components listed here can be reused in external applications.
//!
//! Note that public API is not the main focus of this app crate.
//!
//! See also the `maps_io_ros` crate for a reusable ROS map core library
//! that's used by this app.

pub mod app;
mod app_impl;
mod draw_order;
pub mod error;
mod grid;
mod grid_options;
mod image;
mod image_pyramid;
mod lens;
mod map_pose_ext;
mod map_state;
mod movable;
pub mod os_helpers;
pub mod persistence;
mod rect_helpers;
mod render_options;
mod texture_cache;
mod texture_request;
mod texture_state;
mod tiles;
mod tiles_behavior;
mod tracing;

#[cfg(not(target_arch = "wasm32"))]
pub mod main_native;

#[cfg(target_arch = "wasm32")]
pub mod main_wasm;
#[cfg(target_arch = "wasm32")]
mod wasm;

// Gather build information from build.rs during compile time.
pub(crate) mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

pub(crate) fn build_info_string() -> String {
    format!(
        "maps v{} rev:{}{} | {} | {}",
        built_info::PKG_VERSION,
        built_info::GIT_VERSION.unwrap_or("unknown"),
        if built_info::GIT_DIRTY.unwrap_or(false) {
            "(+ uncommitted changes)"
        } else {
            ""
        },
        built_info::TARGET,
        built_info::PROFILE,
    )
}
