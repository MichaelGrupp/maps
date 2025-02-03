mod kittest_common;

use std::path::PathBuf;

use eframe::egui;
use env_logger;

use kittest_common::*;
use maps::app::{AppOptions, AppState, TintOptions};
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
