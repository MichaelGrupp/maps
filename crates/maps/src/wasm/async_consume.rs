use log::error;

use crate::app::AppState;

impl AppState {
    #[cfg(target_arch = "wasm32")]
    /// Single shot consumption of data fed via websys IO for App::update() passes.
    pub(crate) fn consume_wasm_io(&mut self) {
        let Ok(mut locked_data) = self.data.wasm_io.try_lock() else {
            return;
        };

        if !locked_data.error.is_empty() {
            self.status.error = locked_data.error.clone();
            error!("{}", self.status.error);
            locked_data.error.clear();
        }

        if locked_data.metas.is_empty() && locked_data.map_poses.is_empty() {
            return;
        }

        // We expect metas and images to be in corresponding order here
        // and cheap to clone (Meta, Arc<ImagePyramid>).
        let metas_and_images: Vec<_> = locked_data
            .metas
            .iter()
            .cloned()
            .zip(locked_data.images.iter().cloned())
            .collect();

        let map_poses = locked_data.map_poses.clone();

        locked_data.metas.clear();
        locked_data.images.clear();
        locked_data.map_poses.clear();
        drop(locked_data);

        for (meta, image_pyramid) in metas_and_images {
            let name = meta.yaml_path.to_str().unwrap().to_owned();
            self.add_map(&name, meta, &image_pyramid);
        }

        for (name, map_pose) in map_poses {
            self.add_map_pose(name.as_str(), map_pose);
        }
    }
}
