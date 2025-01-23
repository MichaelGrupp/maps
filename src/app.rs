use std::collections::HashMap;
use std::vec::Vec;

use eframe::egui;
use egui_file_dialog::FileDialog;
use strum_macros::{Display, EnumString, VariantNames};

use crate::app_impl::canvas_settings::CanvasOptions;
use crate::app_impl::pose_edit::PoseEditOptions;
use crate::app_impl::tint_settings::TintOptions;
use crate::grid_options::GridOptions;
use crate::lens::LensOptions;
use crate::map_state::MapState;
use crate::meta::Meta;
use crate::tiles::Tiles;

#[derive(Clone, Debug, Default, PartialEq, Display, EnumString, VariantNames)]
pub enum ViewMode {
    Tiles,
    Stacked,
    #[default]
    Aligned,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum ActiveMovable {
    None,
    MapPose,
    #[default]
    Grid,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum ActiveTool {
    None,
    #[default]
    HoverLens,
    PlaceLens,
    Measure,
}

#[derive(Debug, Default)]
pub struct AppOptions {
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
    pub warning: String,
    pub info: String,
    pub hover_position: Option<egui::Pos2>,
}

#[derive(Default)]
pub struct AppState {
    pub options: AppOptions,
    pub build_info: String,
    pub maps: HashMap<String, MapState>,
    pub grid_lenses: HashMap<String, egui::Pos2>,
    pub status: StatusInfo,
    pub load_meta_file_dialog: FileDialog,
    pub load_map_pose_file_dialog: FileDialog,
    pub save_map_pose_file_dialog: FileDialog,
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

        for meta in metas {
            state.load_image(meta)?;
        }
        for map in state.maps.values_mut() {
            map.tint = Some(state.options.tint_settings.tint_for_all);
        }
        state.load_meta_file_dialog = Self::make_yaml_file_dialog();
        state.load_map_pose_file_dialog = Self::make_yaml_file_dialog();
        state.save_map_pose_file_dialog = Self::make_yaml_file_dialog()
            .allow_file_overwrite(true)
            .default_file_name("map_pose.yaml");

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
            self.handle_key_shortcuts(ui);

            self.header_panel(ui);
            self.menu_panel(ui);
            self.settings_panel(ui);
            self.central_panel(ui);
            self.footer_panel(ui);

            self.info_window(ui);
        });
    }
}
