mod kittest_common;

use std::path::PathBuf;

use eframe::egui;

use kittest_common::*;
use maps::app::{AppOptions, AppState, ViewMode};

const WIKI_SESSION: &str = "tests/sessions/value_interpretations_ros1_wiki.toml";
const MAP_SERVER_SESSION: &str = "tests/sessions/value_interpretations_map_server.toml";

#[test]
fn main() {
    env_logger::init();

    // Snapshot with ROS Wiki value interpretation.
    run("value_interpretations_wiki", WIKI_SESSION);

    // Snapshot with map_server implementation quirks.
    run("value_interpretations_map_server", MAP_SERVER_SESSION);
}

fn run(name: &str, session_file: &str) {
    let mut app_state = AppState::init(
        Vec::new(),
        AppOptions {
            view_mode: ViewMode::Stacked,
            ..Default::default()
        },
    )
    .expect("Failed to initialize AppState");

    app_state.load_session(PathBuf::from(session_file));
    snapshot_full_app(app_state, name, egui::Vec2::new(500., 1000.));
}
