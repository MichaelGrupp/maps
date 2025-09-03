mod kittest_common;

use std::path::PathBuf;

use eframe::egui;

use kittest_common::*;
use maps::app::AppState;
use maps::persistence::load_app_options;

const OPTIONS_PATH: &str = "tests/sessions/custom_options.toml";
const SESSION_PATH: &str = "tests/sessions/custom_session.toml";

#[test]
fn main() {
    env_logger::init();

    let app_options = load_app_options(&Some(PathBuf::from(OPTIONS_PATH)));

    let mut app_state =
        AppState::init(Vec::new(), app_options).expect("Failed to initialize AppState");

    app_state
        .load_session(&PathBuf::from(SESSION_PATH))
        .expect("Failed to load session");

    snapshot_full_app(
        app_state,
        "reload_session_config",
        egui::Vec2::new(1000., 750.),
    );
}
