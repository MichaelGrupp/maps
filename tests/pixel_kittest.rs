mod kittest_common;

use eframe::egui;
use env_logger;

use kittest_common::*;
use maps::app::{AppOptions, AppState, GridOptions};

#[test]
fn main() {
    env_logger::init();

    let app_state = AppState::init(
        // 5 x 5 map with 10 m resolution for testing accurate rendering.
        vec![_load_meta_with_fake_path("data/dummy_maps/pixel_test.yaml")],
        AppOptions {
            grid: GridOptions {
                scale: 10.,
                offset: egui::vec2(-250., 250.),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .expect("Failed to initialize AppState");

    snapshot_full_app(app_state, "pixel_test", egui::Vec2::new(1000., 750.));
}
