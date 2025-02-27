use std::collections::HashMap;
use std::sync::Arc;

use crate::image_pyramid::ImagePyramid;
use crate::map_pose::MapPose;
use crate::meta::Meta;

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
