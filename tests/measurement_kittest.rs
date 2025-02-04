mod kittest_common;

use eframe::egui;
use env_logger;

use kittest_common::*;
use maps::app::{ActiveTool, AppOptions, AppState, GridOptions, TintOptions};

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
            active_tool: ActiveTool::Measure,
            grid: GridOptions {
                measure_start: Some(egui::Pos2::new(-10., -10.)),
                measure_end: Some(egui::Pos2::new(10., 10.)),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .expect("Failed to initialize AppState");

    snapshot_full_app(app_state, "measurement", egui::Vec2::new(1000., 750.));
}
