mod kittest_common;

use eframe::egui;
use env_logger;

use kittest_common::*;
use maps::app::{AppOptions, AppState, TintOptions};

#[test]
fn main() {
    env_logger::init();

    let app_state = AppState::init(
        vec![
            _load_meta_with_fake_path(TEST_META_0),
            _load_meta_with_fake_path(TEST_META_1),
        ],
        AppOptions {
            tint_settings: TintOptions {
                tint_for_all: egui::Color32::from_rgba_unmultiplied(255, 255, 255, 127),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .expect("Failed to initialize AppState");

    snapshot_full_app(app_state, "aligned_view", egui::Vec2::new(1000., 750.));
}
