use std::collections::HashMap;
use std::sync::Arc;

use maps_io_ros::{MapPose, Meta};
use maps_rendering::ImagePyramid;

/// Contains data to be passed between the async websys IO and the app.
/// Held as simple as possible because we don't have complex async operations.
/// Expected to be used under a mutex.
#[cfg(target_arch = "wasm32")]
#[derive(Default)]
pub(crate) struct AsyncData {
    pub metas: Vec<Meta>,
    pub images: Vec<Arc<ImagePyramid>>,
    pub map_poses: HashMap<String, MapPose>,
    pub error: String,
}
