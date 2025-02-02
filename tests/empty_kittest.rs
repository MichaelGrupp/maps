mod kittest_common;

use eframe::egui;

use kittest_common::*;
use maps::app::{AppOptions, AppState};

#[test]
fn empty_start() {
    let app_state = AppState::init(
        vec![],
        AppOptions {
            ..Default::default()
        },
    )
    .expect("Failed to initialize AppState");

    snapshot_full_app(app_state, "empty_start", egui::Vec2::new(1000., 750.));
}
