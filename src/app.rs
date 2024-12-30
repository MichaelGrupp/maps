use std::collections::HashMap;
use std::vec::Vec;

use eframe::egui;
use egui_file_dialog::FileDialog;
use strum_macros::{Display, EnumString, VariantNames};

use crate::grid_options::GridOptions;
use crate::lens::LensOptions;
use crate::map_state::MapState;
use crate::meta::Meta;
use crate::texture_request::NO_TINT;
use crate::tiles::Tiles;

#[derive(Clone, Debug, Default, PartialEq, Display, EnumString, VariantNames)]
pub enum ViewMode {
    Tiles,
    Stacked,
    #[default]
    Aligned,
}

#[derive(Debug, Default)]
pub struct AppOptions {
    pub menu_visible: bool,
    pub settings_visible: bool,
    pub view_mode: ViewMode,
    pub lens: LensOptions,
    pub grid: GridOptions,
    pub active_lens: Option<String>,
    pub active_tint_selection: Option<String>,
    pub tint_for_all: egui::Color32,
}

#[derive(Default)]
pub struct AppState {
    pub options: AppOptions,
    pub maps: HashMap<String, MapState>,
    pub status_message: String,
    pub file_dialog: FileDialog,
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
        state.options.tint_for_all = NO_TINT;

        for meta in metas {
            state.load_image(meta)?;
        }
        state.file_dialog = Self::make_yaml_file_dialog();
        Ok(state)
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
        });
    }
}
