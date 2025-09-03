mod kittest_common;

use std::path::PathBuf;

use eframe::egui;

use kittest_common::*;
use maps::app::{AppOptions, AppState, ColorMap, ViewMode};

const SESSION: &str = "tests/sessions/value_interpretations_map_server.toml";

#[test]
fn main() {
    env_logger::init();

    // Snapshot with ROS map_server value interpretation + RViz "Map" colormap.
    run("value_colormap_rviz_map", ColorMap::RvizMap);

    // Snapshot with map_server value interpretation + RViz "Costmap" colormap.
    run("value_colormap_rviz_costmap", ColorMap::RvizCostmap);
}

fn run(name: &str, colormap: ColorMap) {
    let mut app_state = AppState::init(
        Vec::new(),
        AppOptions {
            view_mode: ViewMode::Stacked,
            ..Default::default()
        },
    )
    .expect("Failed to initialize AppState");

    app_state.load_session(&PathBuf::from(SESSION));

    for map in app_state.data.maps.values_mut() {
        map.meta.value_interpretation.colormap = colormap;
    }

    snapshot_full_app(app_state, name, egui::Vec2::new(500., 1000.));
}
