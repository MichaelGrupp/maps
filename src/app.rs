use std::collections::{BTreeMap, HashMap};
use std::path::{absolute, PathBuf};
use std::vec::Vec;

use eframe::egui;
use egui_file_dialog::FileDialog;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString, VariantNames};

use crate::app_impl::canvas_settings::CanvasOptions;
use crate::app_impl::pose_edit::PoseEditOptions;
use crate::app_impl::tint_settings::TintOptions;
use crate::grid_options::GridOptions;
use crate::lens::LensOptions;
use crate::map_state::MapState;
use crate::meta::Meta;
use crate::persistence::save_app_options;
use crate::tiles::Tiles;

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

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AppOptions {
    pub version: String,
    pub custom_config_path: Option<PathBuf>,
    pub autosave: bool,
    pub canvas_settings: CanvasOptions,
    pub menu_visible: bool,
    pub settings_visible: bool,
    pub help_visible: bool,
    pub view_mode: ViewMode,
    pub lens: LensOptions,
    pub grid: GridOptions,
    pub active_lens: Option<String>,
    pub tint_settings: TintOptions,
    pub pose_edit: PoseEditOptions,
    pub active_movable: ActiveMovable,
    pub active_tool: ActiveTool,
}

#[derive(Default)]
pub struct StatusInfo {
    pub error: String,
    pub hover_position: Option<egui::Pos2>,
}

#[derive(Default)]
pub struct AppState {
    pub options: AppOptions,
    pub build_info: String,
    pub maps: BTreeMap<String, MapState>,
    pub grid_lenses: HashMap<String, egui::Pos2>,
    pub status: StatusInfo,
    pub load_meta_file_dialog: FileDialog,
    pub load_map_pose_file_dialog: FileDialog,
    pub save_map_pose_file_dialog: FileDialog,
    pub load_session_file_dialog: FileDialog,
    pub save_session_file_dialog: FileDialog,
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

        let mut default_dir = None;
        for meta in metas {
            // Use the directory of a meta file as the file dialogs default,
            // this is usually more handy than cwd when file are passed via CLI.
            default_dir = absolute(meta.yaml_path.parent().expect("No parent dir?")).ok();

            state.load_map(meta)?;
        }
        for map in state.maps.values_mut() {
            map.tint = Some(state.options.tint_settings.tint_for_all);
        }
        state.load_meta_file_dialog = Self::make_yaml_file_dialog(&default_dir);
        state.load_map_pose_file_dialog = Self::make_yaml_file_dialog(&default_dir);
        state.save_map_pose_file_dialog = Self::make_yaml_file_dialog(&default_dir)
            .allow_file_overwrite(true)
            .default_file_name("map_pose.yaml");
        state.load_session_file_dialog = Self::make_toml_file_dialog(&default_dir);
        state.save_session_file_dialog = Self::make_toml_file_dialog(&default_dir)
            .allow_file_overwrite(true)
            .default_file_name("maps_session.toml");

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
            self.error_modal(ui);
            self.handle_key_shortcuts(ui);

            self.header_panel(ui);
            self.menu_panel(ui);
            self.settings_panel(ui);
            self.footer_panel(ui);
            self.central_panel(ui);

            self.info_window(ui);
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if !self.options.autosave {
            return;
        }

        // Clear some settings that should not be saved.
        self.options.grid.measure_start = None;
        self.options.grid.measure_end = None;
        self.options.active_tool = ActiveTool::None;

        save_app_options(&self.options);
    }
}
