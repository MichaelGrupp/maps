mod kittest_common;

use std::path::PathBuf;

use eframe::egui;

use kittest_common::*;
use maps::app::{AppOptions, AppState, CanvasOptions, GridOptions, PoseEditOptions};

const DATA_DIR: &str = "data/google_cartographer_example/";
const SESSION_FILENAME: &str = "session.toml";

#[test]
fn main() {
    env_logger::init();

    // File paths in session file are relative to the data directory.
    let start_dir = std::env::current_dir().expect("Failed to get current directory");
    std::env::set_current_dir(DATA_DIR).expect("Failed to go to data directory");

    let mut app_state = AppState::init(
        Vec::new(),
        AppOptions {
            menu_visible: true,
            settings_visible: true,
            canvas_settings: CanvasOptions {
                theme_preference: egui::ThemePreference::Light,
                ..Default::default()
            },
            pose_edit: PoseEditOptions {
                selected_map: "map_1.yaml".to_string(),
                ..Default::default()
            },
            grid: GridOptions {
                scale: 4.,
                offset: egui::vec2(-300., 100.),
                line_spacing_meters: 25.,
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .expect("Failed to initialize AppState");

    app_state
        .load_session(&PathBuf::from(SESSION_FILENAME))
        .expect("Failed to load session");

    // egui_kittest expects us to be in the root directory for saving the snapshot.
    std::env::set_current_dir(start_dir).expect("Failed to go back to start directory");

    snapshot_full_app(
        app_state,
        "panels_visible",
        egui::Vec2::new(1500., 1000.),
    );
}
