use std::path::PathBuf;

use eframe::egui;
use eframe::egui::Context;
use egui_kittest::Harness;

use maps::app::AppState;
use maps::meta::Meta;

// Expects that cargo test is run from the root of the repository.
#[allow(dead_code)]
pub const TEST_META_0: &str = "data/dummy_maps/dummy_map_lores.yaml";
#[allow(dead_code)]
pub const TEST_META_1: &str = "data/dummy_maps/dummy_map_rot.yaml";

const PIXELS_PER_POINT: f32 = 1.;

/// Spins up the full app state UI.
/// Does a snapshot diff test unless the "kittest_snapshots" feature is disabled.
/// To create/update baseline snapshots, run: UPDATE_SNAPSHOTS=1 cargo test
pub fn snapshot_full_app(mut app_state: AppState, test_name: &str, size: egui::Vec2) {
    let app_closure = |ctx: &Context| {
        // This is copypasta from AppState::update().
        // TODO: refactor this once egui_kittest can run eframe::App.
        // Waiting for this to be released:
        // https://github.com/emilk/egui/commit/46b58e5bcca0bb34861b5671958be872419bee90
        egui::CentralPanel::default().show(ctx, |ui| {
            app_state.error_modal(ui);
            app_state.quit_modal(ui);
            app_state.handle_key_shortcuts(ui);

            app_state.header_panel(ui);
            app_state.menu_panel(ui);
            app_state.footer_panel(ui);
            app_state.settings_panel(ui);
            app_state.central_panel(ui);

            app_state.info_window(ui);
        });

        if ctx.input(|i| i.viewport().close_requested()) {
            if app_state.status.unsaved_changes {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                app_state.status.quit_modal_active = true;
            }
        }
    };

    let mut harness = Harness::builder()
        .with_size(size)
        .with_pixels_per_point(PIXELS_PER_POINT)
        .build(app_closure);
    harness.run();

    #[cfg(feature = "kittest_snapshots")]
    harness.wgpu_snapshot(test_name);

    #[cfg(not(feature = "kittest_snapshots"))]
    println!(
        "Snapshot diff test for {} skipped. \
        Enable the 'kittest_snapshots' feature to run it.",
        test_name
    );
}

/// Load the metadata with faked absolute YAML path.
/// Allows to have runner-agnostic snapshots when paths are shown in the UI.
pub fn load_meta_with_fake_path(meta_path: &str) -> Meta {
    let mut meta = Meta::load_from_file(&PathBuf::from(meta_path)).expect("Failed to load map");
    let fake_parent = PathBuf::from("/fake_path_for_testing/");
    meta.yaml_path = fake_parent.join(meta.yaml_path.file_name().unwrap());
    meta
}
