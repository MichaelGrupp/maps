mod kittest_common;

use eframe::egui;
use env_logger;

use kittest_common::*;
use maps::app::{AppOptions, AppState, CanvasOptions, GridOptions, TintOptions};

#[test]
fn main() {
    env_logger::init();

    let mut app_state = AppState::init(
        vec![_load_meta_with_fake_path(TEST_META_0)],
        AppOptions {
            canvas_settings: CanvasOptions {
                background_color: egui::Color32::from_rgba_premultiplied(0, 50, 60, 100),
                ..Default::default()
            },
            tint_settings: TintOptions {
                tint_for_all: egui::Color32::from_rgba_unmultiplied(255, 255, 255, 127),
                ..Default::default()
            },
            grid: GridOptions {
                scale: 3.,
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .expect("Failed to initialize AppState");

    app_state
        .data
        .grid_lenses
        .insert("test".to_string(), egui::pos2(-10., 15.));

    snapshot_full_app(
        app_state,
        "fixed_lens_background",
        egui::Vec2::new(1000., 750.),
    );
}
