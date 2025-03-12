//! Main application state and options.

use std::collections::{BTreeMap, HashMap};
use std::path::absolute;
use std::sync::Arc;
use std::vec::Vec;

use eframe::egui;
use egui_file_dialog::FileDialog;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString, VariantNames};

pub use crate::app_impl::canvas_settings::CanvasOptions;
pub use crate::app_impl::pose_edit::PoseEditOptions;
pub use crate::app_impl::tint_settings::TintOptions;
pub use crate::grid_options::GridOptions;
pub use crate::lens::LensOptions;
pub use crate::value_colormap::ColorMap;

use crate::draw_order::DrawOrder;
use crate::map_state::MapState;
use crate::meta::Meta;
use crate::persistence::{save_app_options, PersistenceOptions};
use crate::tiles::Tiles;

#[cfg(target_arch = "wasm32")]
use crate::wasm::async_data::AsyncData;
#[cfg(target_arch = "wasm32")]
use std::sync::{Arc, Mutex};

#[derive(
    Clone, Debug, Default, PartialEq, Display, EnumString, VariantNames, Serialize, Deserialize,
)]
pub enum ViewMode {
    Tiles,
    Stacked,
    #[default]
    Aligned,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum ActiveMovable {
    None,
    MapPose,
    #[default]
    Grid,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum ActiveTool {
    #[default]
    None,
    HoverLens,
    PlaceLens,
    Measure,
}

/// Contains all configurable options of the application.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AppOptions {
    pub version: String,
    pub persistence: PersistenceOptions,
    pub canvas_settings: CanvasOptions,
    pub menu_visible: bool,
    pub settings_visible: bool,
    pub help_visible: bool,
    pub view_mode: ViewMode,
    pub lens: LensOptions,
    pub grid: GridOptions,
    pub tint_settings: TintOptions,
    pub pose_edit: PoseEditOptions,
    pub active_movable: ActiveMovable,
    pub active_tool: ActiveTool,
}

#[derive(Default)]
pub struct StatusInfo {
    pub error: String,
    pub hover_position: Option<egui::Pos2>,
    pub quit_modal_active: bool,
    pub debug_window_active: bool,
    pub draw_order_edit_active: bool,
    pub unsaved_changes: bool,
    pub quit_after_save: bool,
    pub move_action: Option<String>,
    pub active_tool: Option<String>,
}

#[derive(Default, Serialize, Deserialize)]
pub struct SessionData {
    pub maps: BTreeMap<String, MapState>,
    #[serde(skip)]
    pub draw_order: DrawOrder,
    pub grid_lenses: HashMap<String, egui::Pos2>,
    pub screenshot: Option<Arc<egui::ColorImage>>,

    #[cfg(target_arch = "wasm32")]
    #[serde(skip)]
    pub(crate) wasm_io: Arc<Mutex<AsyncData>>,
    #[cfg(target_arch = "wasm32")]
    #[serde(skip)]
    pub(crate) demo_button_image_handle: Option<egui::TextureHandle>,
}

/// Main application state, implements the `eframe::App` trait.
#[derive(Default)]
pub struct AppState {
    pub options: AppOptions,
    pub build_info: String,
    pub data: SessionData,
    pub status: StatusInfo,
    pub load_meta_file_dialog: FileDialog,
    pub load_map_pose_file_dialog: FileDialog,
    pub save_map_pose_file_dialog: FileDialog,
    pub load_session_file_dialog: FileDialog,
    pub save_session_file_dialog: FileDialog,
    pub save_screenshot_dialog: FileDialog,
    pub tile_manager: Tiles,
}

#[derive(Debug)]
pub struct Error {
    pub message: String,
}

impl AppState {
    pub fn init(metas: Vec<Meta>, options: AppOptions) -> Result<AppState, Error> {
        let mut state = AppState::default();
        state.options = options;

        let mut _default_dir = None;
        for meta in metas {
            // Use the directory of a meta file as the file dialogs default,
            // this is usually more handy than cwd when file are passed via CLI.
            _default_dir = absolute(meta.yaml_path.parent().expect("No parent dir?")).ok();

            state.load_map(meta)?;
        }
        for map in state.data.maps.values_mut() {
            map.tint = Some(state.options.tint_settings.tint_for_all);
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            state.load_meta_file_dialog = Self::make_yaml_file_dialog(&_default_dir);
            state.load_map_pose_file_dialog = Self::make_yaml_file_dialog(&_default_dir);
            state.save_map_pose_file_dialog = Self::make_yaml_file_dialog(&_default_dir)
                .allow_file_overwrite(true)
                .default_file_name("map_pose.yaml");
            state.load_session_file_dialog = Self::make_toml_file_dialog(&_default_dir);
            state.save_session_file_dialog = Self::make_toml_file_dialog(&_default_dir)
                .allow_file_overwrite(true)
                .default_file_name("maps_session.toml");
            state.save_screenshot_dialog =
                Self::make_png_file_dialog(&_default_dir).default_file_name("maps_screenshot.png");
        }

        Ok(state)
    }

    pub fn with_build_info(mut self, build_info: String) -> Self {
        self.build_info = build_info;
        self
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ctx.set_theme(self.options.canvas_settings.theme_preference);

            self.error_modal(ui);
            self.quit_modal(ui);
            self.handle_key_shortcuts(ui);

            self.header_panel(ui);
            self.menu_panel(ui);
            self.footer_panel(ui);
            self.settings_panel(ui);
            self.central_panel(ui);

            self.info_window(ui);
            self.debug_window(ctx, ui);
        });

        self.handle_new_screenshot(&ctx);

        #[cfg(target_arch = "wasm32")]
        self.consume_wasm_io();

        if ctx.input(|i| i.viewport().close_requested())
            && self.status.unsaved_changes
            && !self.data.maps.is_empty()
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.status.quit_modal_active = true;
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if !self.options.persistence.autosave {
            return;
        }

        // Clear some settings that should not be saved.
        self.options.grid.measure_start = None;
        self.options.grid.measure_end = None;
        self.options.active_tool = ActiveTool::None;

        save_app_options(&self.options);
    }
}
