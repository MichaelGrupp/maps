mod kittest_common;

use std::path::PathBuf;

use eframe::egui;

use kittest_common::*;
use maps::app::{AppOptions, AppState, ViewMode};
use maps::meta::Meta;

#[test]
fn main() {
    env_logger::init();

    let app_state = AppState::init(
        vec![
            Meta::load_from_file(&PathBuf::from(TEST_META_0)).expect("Failed to load map"),
            Meta::load_from_file(&PathBuf::from(TEST_META_1)).expect("Failed to load map"),
        ],
        AppOptions {
            view_mode: ViewMode::Tiles,
            ..Default::default()
        },
    )
    .expect("Failed to initialize AppState");

    snapshot_full_app(app_state, "tiles_view", egui::Vec2::new(1000., 750.));
}
