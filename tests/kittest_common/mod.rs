use eframe::egui;
use eframe::egui::Context;
use egui_kittest::Harness;

use maps::app::AppState;

// Expects that cargo test is run from the root of the repository.
#[allow(dead_code)]
pub const TEST_META_0: &str = "data/dummy_maps/dummy_map_lores.yaml";
#[allow(dead_code)]
pub const TEST_META_1: &str = "data/dummy_maps/dummy_map_rot.yaml";

const PIXELS_PER_POINT: f32 = 1.;

/// Does a snapshot diff test of a frame of the full app state.
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

    harness.wgpu_snapshot(test_name);
}
