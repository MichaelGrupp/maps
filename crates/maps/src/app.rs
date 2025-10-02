//! Main application state and options.

use std::collections::{BTreeMap, HashMap};
use std::path::absolute;
use std::vec::Vec;

use eframe::egui;
use egui_file_dialog::FileDialog;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString, VariantNames};

pub use crate::app_impl::canvas_settings::CanvasOptions;
pub use crate::app_impl::pose_edit::PoseEditOptions;
pub use crate::app_impl::tint_settings::TintOptions;
pub use crate::error::{Error, Result};
pub use crate::grid_options::GridOptions;
pub use crate::lens::LensOptions;
pub use maps_io_ros::ColorMap;

use crate::app_impl::CUSTOM_TITLEBAR_SUPPORTED;
use crate::draw_order::DrawOrder;
use crate::map_state::MapState;
use crate::persistence::{PersistenceOptions, save_app_options};
use crate::tiles::Tiles;
use crate::tracing::Tracing;
use maps_io_ros::Meta;
use maps_rendering::render_options::default_crop_threshold;

#[cfg(target_arch = "wasm32")]
use crate::wasm::async_data::AsyncData;
#[cfg(target_arch = "wasm32")]
use std::sync::{Arc, Mutex};

#[cfg(not(target_arch = "wasm32"))]
use crate::app_impl::file_dialog_builder;
#[cfg(not(target_arch = "wasm32"))]
use image::DynamicImage;

#[derive(
    Clone, Debug, Default, PartialEq, Display, EnumString, VariantNames, Serialize, Deserialize,
)]
pub enum ViewMode {
    Tiles,
    Stacked,
    #[default]
    Aligned,
}

#[derive(Clone, Debug, Display, Default, PartialEq, EnumString, VariantNames)]
pub enum ActiveMovable {
    None,
    #[strum(to_string = "Map Pose")]
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

#[derive(Clone, Debug, Default, PartialEq)]
pub enum TitleBar {
    #[default]
    Default,
    Custom,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CollapsedState {
    pub app_settings: bool,
    pub canvas_settings: bool,
    pub tint_settings: bool,
    pub lens_settings: bool,
    pub grid_settings: bool,
    pub tool_settings: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DisplayOptions {
    pub show_full_paths: bool,
}

/// Contains all configurable options of the application.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AppOptions {
    pub version: String,
    pub persistence: PersistenceOptions,
    #[serde(default)]
    pub advanced: AdvancedOptions,
    #[serde(skip)]
    pub titlebar: TitleBar,
    pub canvas_settings: CanvasOptions,
    pub menu_visible: bool,
    pub settings_visible: bool,
    pub help_visible: bool,
    pub view_mode: ViewMode,
    pub lens: LensOptions,
    pub grid: GridOptions,
    #[serde(skip)]
    pub tint_settings: TintOptions,
    #[serde(skip)]
    pub pose_edit: PoseEditOptions,
    #[serde(skip)]
    pub active_movable: ActiveMovable,
    #[serde(skip)]
    pub active_tool: ActiveTool,
    #[serde(default)]
    pub collapsed: CollapsedState,
    #[serde(default)]
    pub display: DisplayOptions,
}

impl AppOptions {
    /// Enables a more compact custom titlebar on platforms that support it.
    /// Shows the app header UI in the titlebar next to the window controls.
    pub fn with_custom_titlebar(mut self) -> Self {
        if CUSTOM_TITLEBAR_SUPPORTED {
            self.titlebar = TitleBar::Custom;
        }
        self
    }

    /// Shall the main window use a custom titlebar?
    pub fn custom_titlebar(&self) -> bool {
        self.titlebar == TitleBar::Custom
    }

    #[cfg(target_arch = "wasm32")]
    pub fn with_dark_theme(mut self) -> Self {
        self.canvas_settings.theme_preference = egui::ThemePreference::Dark;
        self
    }
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
    pub version: Option<String>,
    pub maps: BTreeMap<String, MapState>,
    #[serde(default)]
    pub draw_order: DrawOrder,
    pub grid_lenses: HashMap<String, egui::Pos2>,

    #[cfg(not(target_arch = "wasm32"))]
    #[serde(skip)]
    pub screenshot: Option<DynamicImage>,

    #[cfg(target_arch = "wasm32")]
    #[serde(skip)]
    pub(crate) wasm_io: Arc<Mutex<AsyncData>>,
    #[cfg(target_arch = "wasm32")]
    #[serde(skip)]
    pub(crate) demo_button_image_handle: Option<egui::TextureHandle>,
    #[cfg(target_arch = "wasm32")]
    #[serde(skip)]
    pub(crate) nav2_demo_button_image_handle: Option<egui::TextureHandle>,
}

/// Options that should not need to be changed by the (average) user.
#[derive(Debug, Serialize, Deserialize)]
pub struct AdvancedOptions {
    /// Threshold for cropping large textures in the main grid.
    /// Too low values cause unnecessary cropping (CPU overhead),
    /// too high values lead to too high texture memory usage.
    #[serde(default = "default_crop_threshold")]
    pub grid_crop_threshold: u32,
    /// Dry run mode. E.g. to not actually load images (e.g. in --init-only mode of CLI).
    #[serde(skip)]
    pub dry_run: bool,
}

impl Default for AdvancedOptions {
    fn default() -> Self {
        Self {
            grid_crop_threshold: default_crop_threshold(),
            dry_run: false,
        }
    }
}

/// Main application state, implements the `eframe::App` trait.
#[derive(Default)]
pub struct AppState {
    pub options: AppOptions,
    pub build_info: String,
    pub data: SessionData,
    pub status: StatusInfo,
    pub tracing: Tracing,
    pub load_meta_file_dialog: FileDialog,
    pub load_map_pose_file_dialog: FileDialog,
    pub save_map_pose_file_dialog: FileDialog,
    pub load_session_file_dialog: FileDialog,
    pub save_session_file_dialog: FileDialog,
    pub save_screenshot_dialog: FileDialog,
    pub tile_manager: Tiles,
}

impl AppState {
    pub fn init(metas: Vec<Meta>, options: AppOptions) -> Result<AppState> {
        let mut state = AppState {
            options,
            ..Default::default()
        };
        state.data.version = Some(state.options.version.clone());

        let mut _default_dir = None;
        for meta in metas {
            // Use the directory of a meta file as the file dialogs default,
            // this is usually more handy than cwd when file are passed via CLI.
            _default_dir = absolute(meta.yaml_path.parent().expect("No parent dir?")).ok();

            state.load_map(meta)?;
        }
        for map in state.data.maps.values_mut() {
            map.tint = Some(state.options.tint_settings.tint_for_all);
            if let Some(color_to_alpha) = state.options.tint_settings.color_to_alpha_for_all {
                map.color_to_alpha = Some(color_to_alpha);
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            state.load_meta_file_dialog = file_dialog_builder::yaml(_default_dir.as_ref());
            state.load_map_pose_file_dialog = file_dialog_builder::yaml(_default_dir.as_ref());
            state.save_map_pose_file_dialog = file_dialog_builder::yaml(_default_dir.as_ref())
                .allow_file_overwrite(true)
                .default_file_name("map_pose.yaml");
            state.load_session_file_dialog = file_dialog_builder::toml(_default_dir.as_ref());
            state.save_session_file_dialog = file_dialog_builder::toml(_default_dir.as_ref())
                .allow_file_overwrite(true)
                .default_file_name("maps_session.toml");
            state.save_screenshot_dialog = file_dialog_builder::png(_default_dir.as_ref())
                .default_file_name("maps_screenshot.png");
        }

        const TRACING_BUFFER_SIZE: usize = 600;
        state.tracing = Tracing::new("frame update", TRACING_BUFFER_SIZE);

        Ok(state)
    }

    pub fn with_build_info(mut self, build_info: String) -> Self {
        self.build_info = build_info;
        self
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.tracing.start();

        let mut central_rect = egui::Rect::ZERO;

        egui::CentralPanel::default().show(ctx, |ui| {
            ctx.set_theme(self.options.canvas_settings.theme_preference);

            self.error_modal(ui);
            self.quit_modal(ui);
            self.handle_key_shortcuts(ui);

            self.header_panel(ui);
            self.menu_panel(ui);
            self.footer_panel(ui);
            self.settings_panel(ui);
            central_rect = self.central_panel(ui);

            self.info_window(ui);
            self.debug_window(ctx, ui);
        });

        self.handle_new_screenshot(ctx, &central_rect);

        #[cfg(target_arch = "wasm32")]
        self.consume_wasm_io();

        if ctx.input(|i| i.viewport().close_requested())
            && self.status.unsaved_changes
            && !self.data.maps.is_empty()
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.status.quit_modal_active = true;
        }

        self.tracing.measure();
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if !self.options.persistence.autosave {
            return;
        }
        save_app_options(&self.options);
    }
}
