mod kittest_common;

use eframe::egui;
use env_logger;

use kittest_common::*;
use maps::app::{AppOptions, AppState, GridOptions};

/// Tests grid main/sub line precision with fine spacing and offset far from origin.
#[test]
fn main() {
    env_logger::init();

    // Scale is on purpose set to a non-integer value to test precision.
    // This was causing issues with distant, dense sub lines in the past.
    let scale = 254.123;
    let metric_offset = egui::vec2(800., 600.);

    let mut app_state = AppState::init(
        vec![_load_meta_with_fake_path(TEST_META_0)],
        AppOptions {
            grid: GridOptions {
                scale,
                offset: metric_offset * scale,
                line_spacing_meters: 0.1,
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .expect("Failed to initialize AppState");

    // Add a fixed lens positioned far from the origin to test also precision of sub lines.
    app_state
        .data
        .grid_lenses
        .insert("test".to_string(), metric_offset.to_pos2());

    snapshot_full_app(
        app_state,
        "dense_grid_precision",
        egui::Vec2::new(1000., 750.),
    );
}
