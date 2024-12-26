use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::vec::Vec;

use eframe::egui;
use egui_file_dialog::FileDialog;
use strum_macros::{Display, EnumString, VariantNames};

use crate::grid::Grid;
use crate::image::load_image;
use crate::image_pyramid::ImagePyramid;
use crate::lens::Lens;
use crate::map_state::MapState;
use crate::meta::Meta;
use crate::texture_state::TextureState;
use crate::tiles::{Pane, Tiles};
use crate::tiles_behavior::MapsTreeBehavior;

const SPACE: f32 = 10.;
const ICON_SIZE: f32 = 20.;

#[derive(Clone, Debug, Default, PartialEq, Display, EnumString, VariantNames)]
pub enum ViewMode {
    Tiles,
    #[default]
    Stacked,
    Aligned,
}

#[derive(Debug, Default)]
pub struct AppOptions {
    pub menu_visible: bool,
    pub settings_visible: bool,
    pub view_mode: ViewMode,
    pub grid_scale: f32,
    pub grid_lines_visible: bool,
    pub grid_spacing: f32,
}

#[derive(Default)]
pub struct AppState {
    options: AppOptions,
    maps: HashMap<String, MapState>,
    lens: Lens,
    status_message: String,
    file_dialog: FileDialog,
    tile_manager: Tiles,
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
        state.file_dialog = FileDialog::new()
            .add_file_filter(
                "yaml",
                Arc::new(|path| {
                    ["yml", "yaml"].contains(
                        &path
                            .extension()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or_default(),
                    )
                }),
            )
            .default_file_filter("yaml");
        Ok(state)
    }

    fn load_meta(&mut self, yaml_path: PathBuf) -> Result<bool, Error> {
        match Meta::load_from_file(yaml_path) {
            Ok(meta) => match self.load_image(meta) {
                Ok(_) => Ok(true),
                Err(e) => Err(e),
            },
            Err(e) => Err(Error {
                message: format!("Error loading metadata file: {:?}", e),
            }),
        }
    }

    fn load_meta_button(&mut self, ui: &mut egui::Ui) {
        if ui.button("📂 Load Maps").clicked() {
            self.file_dialog.pick_multiple();
        }
        self.file_dialog.update(ui.ctx());

        if let Some(paths) = self.file_dialog.take_picked_multiple() {
            for path in paths {
                ui.ctx().request_repaint();
                match self.load_meta(path.clone()) {
                    Ok(_) => {
                        self.status_message = format!("Loaded metadata file: {:?}", path);
                    }
                    Err(e) => {
                        self.status_message =
                            format!("Error loading metadata file: {:?}", e.message);
                    }
                }
            }
        }
    }

    fn load_image(&mut self, meta: Meta) -> Result<(), Error> {
        self.status_message = format!("Loading image: {:?}", meta.image_path);
        match load_image(&meta.image_path) {
            Ok(image) => {
                self.tile_manager.add_pane(Pane {
                    id: meta.yaml_path.to_str().unwrap().to_owned(),
                });
                let image_pyramid = ImagePyramid::new(image);
                self.maps.insert(
                    meta.yaml_path.to_str().unwrap().to_owned(),
                    MapState {
                        meta,
                        visible: true,
                        texture_state: TextureState::new(image_pyramid),
                        overlay_texture: None,
                    },
                );
                Ok(())
            }
            Err(e) => Err(Error {
                message: format!("Error loading image: {:?}", e),
            }),
        }
    }

    fn handle_key_shortcuts(&mut self, ui: &egui::Ui) {
        ui.input(|i| {
            if i.key_released(egui::Key::Escape) {
                self.options.menu_visible = false;
                self.options.settings_visible = false;
                self.lens.enabled = false;
            } else if i.key_released(egui::Key::L) || i.pointer.secondary_released() {
                self.lens.enabled = !self.lens.enabled;
            }
            if i.key_released(egui::Key::M) {
                self.options.menu_visible = !self.options.menu_visible;
            }
            if i.key_released(egui::Key::S) {
                self.options.settings_visible = !self.options.settings_visible;
            }
        });
    }

    fn show_stacked_images(&mut self, ui: &mut egui::Ui) {
        for (name, map) in self.maps.iter_mut() {
            map.texture_state.update_to_available_space(ui, name);
            if !map.visible {
                continue;
            }
            ui.with_layout(egui::Layout::top_down(egui::Align::TOP), |ui| {
                Self::show_image(ui, name, map);
            });
        }
    }

    fn show_image(ui: &mut egui::Ui, name: &str, map: &mut MapState) {
        let texture = match &map.texture_state.texture_handle {
            Some(texture) => texture,
            None => {
                panic!("Missing texture handle for image {}", name);
            }
        };
        map.texture_state.image_response = Some(ui.image(texture));
    }

    fn header_panel(&mut self, ui: &mut egui::Ui) {
        let add_toggle_button = |ui: &mut egui::Ui,
                                 icon: &str,
                                 tooltip: &str,
                                 switch: &mut bool| {
            if ui
                .add_sized(
                    egui::vec2(ICON_SIZE, ICON_SIZE),
                    egui::SelectableLabel::new(*switch, egui::RichText::new(icon).size(ICON_SIZE)),
                )
                .on_hover_text(tooltip)
                .clicked()
            {
                *switch = !*switch;
            }
        };

        egui::TopBottomPanel::new(egui::containers::panel::TopBottomSide::Top, "header").show(
            ui.ctx(),
            |ui| {
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                        add_toggle_button(ui, "☰", "Show Menu", &mut self.options.menu_visible);
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        add_toggle_button(
                            ui,
                            "⚙",
                            "Show Settings",
                            &mut self.options.settings_visible,
                        );
                    });
                });
            },
        );
    }

    fn menu_panel(&mut self, ui: &mut egui::Ui) {
        if !self.options.menu_visible {
            return;
        }
        egui::SidePanel::left("menu").show(ui.ctx(), |ui| {
            ui.heading("Maps");
            ui.add_space(SPACE);
            self.load_meta_button(ui);
            ui.separator();
            let mut to_delete: Vec<String> = Vec::new();
            egui::Grid::new("maps_list")
                .num_columns(2)
                .striped(true)
                .show(ui, |ui| {
                    for (name, map) in &mut self.maps {
                        if ui.checkbox(&mut map.visible, name).changed() {
                            self.tile_manager.set_visible(name, map.visible);
                        }
                        if ui.button("🗑").on_hover_text("Delete Map").clicked() {
                            to_delete.push(name.clone());
                        }
                        ui.end_row();
                    }
                });
            for name in to_delete {
                self.maps.remove(&name);
                self.tile_manager.remove_pane(&name);
            }
        });
    }

    fn settings_panel(&mut self, ui: &mut egui::Ui) {
        if !self.options.settings_visible {
            return;
        }
        egui::SidePanel::right("settings").show(ui.ctx(), |ui| {
            ui.heading("Settings");
            ui.add_space(SPACE);
            egui::Grid::new("settings_grid")
                .num_columns(2)
                .striped(false)
                .show(ui, |ui| {
                    ui.label("View Mode");
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.options.view_mode, ViewMode::Tiles, "Tiles")
                            .on_hover_text(
                                "Show the maps in separate tab tiles that can be rearranged.",
                            );
                        ui.selectable_value(
                            &mut self.options.view_mode,
                            ViewMode::Stacked,
                            "Stacked",
                        )
                        .on_hover_text("Show the maps stacked on top of each other.");
                        ui.selectable_value(
                            &mut self.options.view_mode,
                            ViewMode::Aligned,
                            "Aligned",
                        )
                        .on_hover_text("Show the maps in a shared coordinate system.");
                    });
                    ui.end_row();
                    ui.end_row();

                    ui.heading("Lens");
                    ui.end_row();
                    ui.checkbox(&mut self.lens.enabled, "Show Lens");
                    ui.end_row();
                    ui.label("Lens size (meters)");
                    ui.add(egui::Slider::new(
                        &mut self.lens.size_meters,
                        self.lens.size_meters_min..=self.lens.size_meters_max,
                    ));
                    ui.end_row();
                    ui.label("Scroll speed factor");
                    ui.add(egui::Slider::new(
                        &mut self.lens.scroll_speed_factor,
                        0.0..=1.0,
                    ));
                    ui.end_row();
                    ui.end_row();

                    if self.options.view_mode == ViewMode::Aligned {
                        ui.heading("Grid");
                        ui.end_row();
                        ui.checkbox(&mut self.options.grid_lines_visible, "Show Grid Lines");
                        ui.end_row();
                        ui.label("Grid scale (points per meter)");
                        ui.add(egui::Slider::new(&mut self.options.grid_scale, 1.0..=10.));
                        ui.end_row();
                        ui.label("Grid lines spacing (meters)");
                        ui.add(egui::Slider::new(
                            &mut self.options.grid_spacing,
                            0.1..=100.,
                        ));
                    }
                });
        });
    }

    fn footer_panel(&mut self, ui: &mut egui::Ui) {
        egui::TopBottomPanel::new(egui::containers::panel::TopBottomSide::Bottom, "footer").show(
            ui.ctx(),
            |ui| {
                ui.horizontal(|ui| ui.label(self.status_message.clone()));
            },
        );
    }

    fn central_panel(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui.ctx(), |ui| {
            if self.maps.is_empty() {
                ui.with_layout(
                    egui::Layout::centered_and_justified(egui::Direction::TopDown),
                    |ui| {
                        ui.horizontal_centered(|ui| {
                            ui.vertical_centered(|ui| {
                                ui.add_space((ui.available_height() / 2. - 100.).max(SPACE));
                                ui.heading("No maps loaded.");
                                ui.add_space(SPACE);
                                self.load_meta_button(ui);
                            });
                        });
                    },
                );
                return;
            }

            match self.options.view_mode {
                ViewMode::Tiles => {
                    // TODO: don't initialize the behavior every frame?
                    let mut behavior = MapsTreeBehavior {
                        maps: &mut self.maps,
                    };
                    self.tile_manager.tree.ui(&mut behavior, ui);
                }
                ViewMode::Stacked => {
                    egui::ScrollArea::both().show(ui, |ui| {
                        self.show_stacked_images(ui);
                        // Fill the remaining vertical space, otherwise the scroll bar can jump around.
                        ui.add_space(ui.available_height());
                    });
                }
                ViewMode::Aligned => {
                    let grid = Grid::new(ui, self.options.grid_scale);
                    grid.show_maps(ui, &mut self.maps);
                    if self.options.grid_lines_visible {
                        grid.draw(
                            ui,
                            self.options.grid_spacing,
                            egui::Stroke::new(1.0, egui::Color32::LIGHT_BLUE),
                        );
                    }
                }
            }
            for (name, map) in &mut self.maps {
                self.lens.show_on_hover(ui, map, name);
            }
        });
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
