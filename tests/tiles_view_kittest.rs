mod kittest_common;

use eframe::egui;

use kittest_common::*;
use maps::app::{AppOptions, AppState, ViewMode};

#[test]
fn main() {
    env_logger::init();

    let app_state = AppState::init(
        vec![
            load_meta_with_fake_path(TEST_META_0),
            load_meta_with_fake_path(TEST_META_1),
        ],
        AppOptions {
            view_mode: ViewMode::Tiles,
            ..Default::default()
        },
    )
    .expect("Failed to initialize AppState");

    snapshot_full_app(app_state, "tiles_view", egui::Vec2::new(1000., 750.));
}
