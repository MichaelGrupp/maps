use std::path::PathBuf;

use eframe::egui;
use egui_kittest::HarnessBuilder;

use maps::app::AppState;
use maps::meta::Meta;

// Expects that cargo test is run from the root of the repository.
#[allow(dead_code)]
pub const TEST_META_0: &str = "data/dummy_maps/dummy_map_lores.yaml";
#[allow(dead_code)]
pub const TEST_META_1: &str = "data/dummy_maps/dummy_map_rot.yaml";

const PIXELS_PER_POINT: f32 = 1.;

/// Spins up the full app state UI.
/// Does a snapshot diff test unless the "kittest_snapshots" feature is disabled.
/// To create/update baseline snapshots, run: UPDATE_SNAPSHOTS=1 cargo test
pub fn snapshot_full_app(app_state: AppState, test_name: &str, size: egui::Vec2) {
    if cfg!(feature = "kittest_snapshots") {
        let mut harness = HarnessBuilder::default()
            .with_size(size)
            .with_pixels_per_point(PIXELS_PER_POINT)
            .build_eframe(|_cc: &mut eframe::CreationContext| app_state);

        harness.snapshot(test_name);
    } else {
        println!(
            "Snapshot diff test for {test_name} skipped. \
            Enable the 'kittest_snapshots' feature to run it."
        );
    }
}

/// Load the metadata with faked absolute YAML path.
/// Allows to have runner-agnostic snapshots when paths are shown in the UI.
pub fn _load_meta_with_fake_path(meta_path: &str) -> Meta {
    let mut meta = Meta::load_from_file(&PathBuf::from(meta_path)).expect("Failed to load map");
    let fake_parent = PathBuf::from("/fake_path_for_testing/");
    meta.yaml_path = fake_parent.join(meta.yaml_path.file_name().expect("huh?"));
    meta
}
