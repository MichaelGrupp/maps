use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::vec::Vec;

use eframe::egui;
use egui_file_dialog::FileDialog;
use strum_macros::{Display, EnumString, VariantNames};

use crate::grid::Grid;
use crate::grid_options::GridOptions;
use crate::image::load_image;
use crate::image_pyramid::ImagePyramid;
use crate::lens::{Lens, LensOptions};
use crate::map_state::MapState;
use crate::meta::Meta;
use crate::texture_request::{TextureRequest, NO_TINT};
use crate::texture_state::TextureState;
use crate::tiles::{Pane, Tiles};
use crate::tiles_behavior::MapsTreeBehavior;

const SPACE: f32 = 10.;
const ICON_SIZE: f32 = 20.;

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
    options: AppOptions,
    maps: HashMap<String, MapState>,
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
        state.options.tint_for_all = NO_TINT;

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
                        tint: None,
                    },
                );
                Ok(())
            }
            Err(e) => Err(Error {
                message: format!("Error loading image: {:?}", e),
            }),
        }
    }

    fn flip_lens(&mut self) {
        if !self.options.lens.enabled || self.options.view_mode != ViewMode::Aligned {
            // Activate in all cases, or toggle in non-overlapping views.
            self.options.lens.enabled = !self.options.lens.enabled;
            return;
        }
        // In overlapping aligned view, switch to the next visible maps, then disable.
        let keys: Vec<&String> = self
            .maps
            .keys()
            .filter(|k| self.maps.get(*k).unwrap().visible)
            .collect();
        if keys.is_empty() {
            self.options.lens.enabled = false;
            return;
        }
        let active_lens = self.options.active_lens.get_or_insert(keys[0].clone());
        if let Some(active_index) = keys.iter().position(|k| *k == active_lens) {
            let next_index = (active_index + 1) % keys.len();
            if next_index == 0 {
                self.options.active_lens = None;
                self.options.lens.enabled = false;
            } else {
                *active_lens = keys[next_index].clone();
            }
        }
    }

    fn handle_key_shortcuts(&mut self, ui: &egui::Ui) {
        ui.input(|i| {
            if i.key_released(egui::Key::Escape) {
                self.options.menu_visible = false;
                self.options.settings_visible = false;
                self.options.lens.enabled = false;
            } else if i.key_released(egui::Key::L) || i.pointer.secondary_released() {
                self.flip_lens();
            }
            if i.key_released(egui::Key::M) {
                self.options.menu_visible = !self.options.menu_visible;
            }
            if i.key_released(egui::Key::O) {
                self.options.settings_visible = !self.options.settings_visible;
            }
            if i.key_released(egui::Key::G) {
                self.options.grid.lines_visible = !self.options.grid.lines_visible;
            }
            if i.key_down(egui::Key::W) {
                self.options.grid.offset.y -= 10.;
            }
            if i.key_down(egui::Key::A) {
                self.options.grid.offset.x -= 10.;
            }
            if i.key_down(egui::Key::S) {
                self.options.grid.offset.y += 10.;
            }
            if i.key_down(egui::Key::D) {
                self.options.grid.offset.x += 10.;
            }
            if i.key_down(egui::Key::Minus) {
                self.options.grid.scale = (self.options.grid.scale - 1.)
                    .clamp(self.options.grid.min_scale, self.options.grid.max_scale);
            }
            if i.key_down(egui::Key::Plus) {
                self.options.grid.scale = (self.options.grid.scale + 1.)
                    .clamp(self.options.grid.min_scale, self.options.grid.max_scale);
            }
        });
    }

    fn show_stacked_images(&mut self, ui: &mut egui::Ui) {
        let num_visible = self.maps.values().filter(|m| m.visible).count();
        let rect_per_image = egui::Rect::from_min_max(
            egui::Pos2::ZERO,
            egui::pos2(
                ui.available_width(),
                ui.available_height() / num_visible as f32,
            ),
        );
        for (name, map) in self.maps.iter_mut() {
            if !map.visible {
                continue;
            }
            ui.with_layout(egui::Layout::top_down(egui::Align::TOP), |ui| {
                map.texture_state.put(
                    ui,
                    &TextureRequest::new(name.clone(), rect_per_image).with_tint(map.tint),
                );
            });
        }
    }

    fn view_buttons(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_centered(|ui| {
            ui.selectable_value(&mut self.options.view_mode, ViewMode::Tiles, "Tiles")
                .on_hover_text("Show the maps in separate tab tiles that can be rearranged.");
            ui.selectable_value(&mut self.options.view_mode, ViewMode::Stacked, "Stacked")
                .on_hover_text("Show the maps stacked on top of each other.");
            ui.selectable_value(&mut self.options.view_mode, ViewMode::Aligned, "Aligned")
                .on_hover_text("Show the maps in a shared coordinate system.");
        });
    }

    fn lens_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("Lens");
        if ui.button("Reset").clicked() {
            self.options.lens = LensOptions::default();
        }
        ui.add_space(SPACE);
        ui.end_row();
        ui.label("Show Lens");
        ui.checkbox(&mut self.options.lens.enabled, "");
        ui.end_row();
        ui.label("Lens size (meters)");
        ui.add(egui::Slider::new(
            &mut self.options.lens.size_meters,
            self.options.lens.size_meters_min..=self.options.lens.size_meters_max,
        ));
        ui.end_row();
        ui.label("Zoom speed")
            .on_hover_text("How fast the lens zooms in/out when scrolling.");
        ui.add(egui::Slider::new(
            &mut self.options.lens.scroll_speed_factor,
            0.0..=1.0,
        ));
    }

    fn grid_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("Grid");
        if ui.button("Reset").clicked() {
            self.options.grid = GridOptions::default();
        }
        ui.add_space(SPACE);
        ui.end_row();
        ui.label("Show Grid Lines");
        ui.checkbox(&mut self.options.grid.lines_visible, "");
        ui.end_row();
        ui.label("Grid color");
        ui.color_edit_button_srgba(&mut self.options.grid.line_stroke.color);
        ui.end_row();
        ui.label("Grid lines spacing (meters)");
        ui.add(egui::Slider::new(
            &mut self.options.grid.line_spacing,
            self.options.grid.min_line_spacing..=self.options.grid.max_line_spacing,
        ));
        ui.end_row();
        ui.end_row();
        ui.label("Show marker");
        ui.checkbox(&mut self.options.grid.marker_visible, "");
        ui.end_row();
        ui.label("Marker length (meters)");
        ui.add(egui::Slider::new(
            &mut self.options.grid.marker_length_meters,
            0.1..=25.0,
        ));
        ui.end_row();
        ui.label("Marker width (meters)");
        ui.add(egui::Slider::new(
            &mut self.options.grid.marker_width_meters,
            0.01..=5.,
        ));
        ui.end_row();
        ui.label("Marker color (X)");
        ui.color_edit_button_srgba(&mut self.options.grid.marker_x_color);
        ui.end_row();
        ui.label("Marker color (Y)");
        ui.color_edit_button_srgba(&mut self.options.grid.marker_y_color);
        ui.end_row();
        ui.label("Marker color (Z)");
        ui.color_edit_button_srgba(&mut self.options.grid.marker_z_color);
        ui.end_row();
        ui.end_row();
        ui.label("Grid scale (points per meter)");
        ui.add(egui::Slider::new(
            &mut self.options.grid.scale,
            self.options.grid.min_scale..=self.options.grid.max_scale,
        ));
        ui.end_row();
        ui.label("Zoom speed")
            .on_hover_text("How fast the grid zooms in/out when scrolling.");
        ui.add(egui::Slider::new(
            &mut self.options.grid.scroll_speed_factor,
            0.0..=1.0,
        ));
    }

    fn tint_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("Blend");
        ui.add_space(SPACE);
        ui.end_row();

        let all_key = "< All >".to_string();
        let selected = self
            .options
            .active_tint_selection
            .get_or_insert(all_key.clone());
        egui::ComboBox::from_label("")
            .selected_text(format!("{}", selected))
            .show_ui(ui, |ui| {
                ui.selectable_value(selected, all_key.clone(), &all_key);
                for name in self.maps.keys() {
                    ui.selectable_value(selected, name.to_string(), name);
                }
            });

        let reset = ui.button("Reset").clicked();
        ui.end_row();

        ui.label("Tint color / alpha");
        if *selected == all_key {
            let tint = &mut self.options.tint_for_all;
            if reset {
                *tint = NO_TINT;
            }
            ui.color_edit_button_srgba(tint);
            for map in self.maps.values_mut() {
                map.tint = Some(*tint);
            }
        } else {
            let tint = self
                .maps
                .get_mut(selected)
                .unwrap()
                .tint
                .get_or_insert(NO_TINT);
            if reset {
                *tint = NO_TINT;
            }
            ui.color_edit_button_srgba(tint);
        }
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
                        ui.add_space(ICON_SIZE);
                        self.view_buttons(ui);
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
                if let Some(active_lens) = &self.options.active_lens {
                    if active_lens == &name {
                        self.options.active_lens = None;
                    }
                }
                if let Some(active_tint_selection) = &self.options.active_tint_selection {
                    if active_tint_selection == &name {
                        self.options.active_tint_selection = None;
                    }
                }
            }
        });
    }

    fn settings_panel(&mut self, ui: &mut egui::Ui) {
        if !self.options.settings_visible {
            return;
        }
        egui::SidePanel::right("settings").show(ui.ctx(), |ui| {
            egui::Grid::new("settings_grid")
                .num_columns(2)
                .striped(false)
                .show(ui, |ui| {
                    self.lens_settings(ui);

                    if self.options.view_mode == ViewMode::Aligned {
                        ui.end_row();
                        ui.end_row();
                        self.grid_settings(ui);
                    }

                    if !self.maps.is_empty() {
                        ui.end_row();
                        ui.end_row();
                        self.tint_settings(ui);
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
                    // Modify the grid with the mouse, but only if inside this panel rect.
                    let options = &mut self.options.grid;
                    if ui.rect_contains_pointer(ui.available_rect_before_wrap()) {
                        ui.input(|i| {
                            if i.pointer.primary_down() {
                                options.offset += i.pointer.delta();
                            }
                            if !self.options.lens.enabled {
                                options.scale +=
                                    i.smooth_scroll_delta.y * options.scroll_speed_factor;
                                options.scale =
                                    options.scale.clamp(options.min_scale, options.max_scale);
                            }
                        });
                    }

                    let grid = Grid::new(ui, options.scale).with_origin_offset(options.offset);
                    grid.show_maps(ui, &mut self.maps);
                    if options.lines_visible {
                        grid.draw(ui, options.line_spacing, options.line_stroke);
                    }
                    if options.marker_visible {
                        grid.draw_axes(ui, &options);
                    }
                }
            }
            let num_visible_maps = self.maps.values().filter(|m| m.visible).count();
            for (name, map) in &mut self.maps {
                if !self.options.lens.enabled {
                    continue;
                }
                if self.options.view_mode == ViewMode::Aligned && num_visible_maps > 1 {
                    // Show lens on hover only for the active map in Aligned view mode.
                    let active_lens = self.options.active_lens.get_or_insert(name.to_string());
                    if *active_lens != *name {
                        continue;
                    }
                }
                Lens::with(&mut self.options.lens).show_on_hover(ui, map, name);
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
