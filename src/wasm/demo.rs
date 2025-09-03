use std::sync::Arc;

use eframe::egui;

use crate::app::AppState;
use crate::app_impl::constants::SPACE;
use crate::grid_options::MarkerVisibility;
use crate::image::{load_image_from_bytes, to_egui_image};
use crate::image_pyramid::ImagePyramid;
use crate::map_pose::MapPose;
use crate::meta::Meta;
use crate::movable::Draggable;
use crate::value_colormap::ColorMap;
use crate::value_interpretation;

// Helper function to create a demo button with image.
#[cfg(target_arch = "wasm32")]
pub(crate) fn demo_button(
    ui: &mut egui::Ui,
    button_handle: &mut Option<egui::TextureHandle>,
    texture_name: &str,
    image_bytes: &[u8],
    alt_text: &str,
) -> bool {
    if button_handle.is_none() {
        *button_handle = Some(ui.ctx().load_texture(
            texture_name,
            to_egui_image(load_image_from_bytes(image_bytes).expect("broken demo")),
            Default::default(),
        ));
    }
    ui.add(
        egui::ImageButton::new(
            egui::Image::new(button_handle.as_ref().expect("missing demo image"))
                .max_height(200.)
                .alt_text(alt_text),
        )
        .corner_radius(5.),
    )
    .on_hover_text(alt_text)
    .clicked()
}

// Load the cartographer demo maps.
#[cfg(target_arch = "wasm32")]
fn load_cartographer_demo(app_state: &mut AppState) {
    let tints = [
        egui::Color32::from_rgba_premultiplied(123, 45, 0, 128),
        egui::Color32::from_rgba_premultiplied(32, 67, 32, 74),
        egui::Color32::from_rgba_premultiplied(97, 52, 76, 97),
    ];

    // Macro for loading the cartographer maps at compile time.
    // Note that we use JPEG version here for smaller wasm size.
    macro_rules! load_map {
        ($index:literal) => {{
            let name = format!("map_{}.yaml", $index);
            let img = Arc::new(ImagePyramid::new(
                load_image_from_bytes(include_bytes!(concat!(
                    "../../data/google_cartographer_example/jpeg/map_",
                    stringify!($index),
                    ".jpg"
                )))
                .expect("broken demo"),
            ));
            let meta = Meta::load_from_bytes(
                include_bytes!(concat!(
                    "../../data/google_cartographer_example/map_",
                    stringify!($index),
                    ".yaml"
                )),
                &name,
            )
            .expect("broken demo");
            app_state.add_map(&name, meta, img);

            let map = app_state
                .data
                .maps
                .get_mut(&name)
                .expect("missing demo map");
            map.tint = Some(tints[$index]);
            map.color_to_alpha = Some(egui::Color32::from_gray(128));
            map.pose = MapPose::from_bytes(include_bytes!(concat!(
                "../../data/google_cartographer_example/pose_",
                stringify!($index),
                ".yaml"
            )))
            .expect("broken demo");
        }};
    }

    // Load maps 0, 1, and 2.
    load_map!(0);
    load_map!(1);
    load_map!(2);

    app_state.options.tint_settings.active_tint_selection = Some("map_2.yaml".to_owned());
    app_state.options.grid.scale = 5.5;
    app_state.options.grid.drag(egui::vec2(-75., 30.));
    app_state.options.grid.line_spacing_meters = 25.;
    app_state.options.grid.marker_visibility = MarkerVisibility::All;
    app_state.options.grid.marker_length_meters = 5.;
    app_state.options.grid.marker_width_meters = 0.75;
    app_state.options.canvas_settings.theme_preference = egui::ThemePreference::Dark;

    // Collapse some settings to be less overwhelming and to show that it's possible.
    // (show grid settings expanded)
    app_state.options.collapsed.app_settings = true;
    app_state.options.collapsed.canvas_settings = true;
    app_state.options.collapsed.tool_settings = true;
}

// Embedded demo function with dummy maps (included during compile time) for the wasm target.
// This is a simple way to provide a demo without the need for a server.
#[cfg(target_arch = "wasm32")]
impl AppState {
    pub(crate) fn demo_buttons(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            self.load_demo_nav2_button(ui);
            ui.add_space(2. * SPACE);
            self.load_demo_maps_button(ui);
        });
    }

    pub(crate) fn load_demo_maps_button(&mut self, ui: &mut egui::Ui) {
        if demo_button(
            ui,
            &mut self.data.demo_button_image_handle,
            "_demo_button",
            include_bytes!("../../data/doc/demo_scaled.png"),
            "Demo showcasing very large map images, with maps of the Cartographer Deutsches Museum dataset.",
        ) {
            load_cartographer_demo(self);
        }
    }

    pub(crate) fn load_demo_nav2_button(&mut self, ui: &mut egui::Ui) {
        if demo_button(
            ui,
            &mut self.data.nav2_demo_button_image_handle,
            "_nav2_demo_button",
            include_bytes!("../../data/doc/demo_nav2_scaled.png"),
            "Demo showcasing a typical mobile robot warehouse scenario, with maps from the Nav2 demo.",
        ) {
            load_nav2_demo(self);
        }
    }
}

/// Configuration for loading a nav2 example map.
#[cfg(target_arch = "wasm32")]
struct Nav2MapConfig<'a> {
    name: &'a str,
    image_bytes: &'a [u8],
    yaml_bytes: &'a [u8],
    pose_bytes: &'a [u8],
    value_mode: Option<value_interpretation::Mode>,
    colormap: Option<ColorMap>,
    color_to_alpha: Option<egui::Color32>,
}

/// Helper function to load a single nav2 example map.
#[cfg(target_arch = "wasm32")]
fn load_nav2_map(app_state: &mut AppState, config: Nav2MapConfig) {
    let name = config.name.to_string();
    let img = Arc::new(ImagePyramid::new(
        load_image_from_bytes(config.image_bytes).expect("broken nav2 demo"),
    ));

    let mut meta = Meta::load_from_bytes(config.yaml_bytes, &name).expect("broken nav2 demo");

    // Set up value interpretation if specified.
    if let Some(mode) = config.value_mode {
        meta.value_interpretation.mode = mode;
        meta.value_interpretation.explicit_mode = true;
    }
    if let Some(cmap) = config.colormap {
        meta.value_interpretation.colormap = cmap;
    }

    app_state.add_map(&name, meta, img);

    let map = app_state
        .data
        .maps
        .get_mut(&name)
        .expect("missing nav2 demo map");
    map.tint = Some(egui::Color32::from_rgba_premultiplied(255, 255, 255, 255));
    if let Some(alpha_color) = config.color_to_alpha {
        map.color_to_alpha = Some(alpha_color);
    }
    map.pose = MapPose::from_bytes(config.pose_bytes).expect("broken nav2 demo");
    map.use_value_interpretation = config.value_mode.is_some();
}

/// Embeds all nav2 demo maps like in the native app session file
/// (data/nav2_example/session.toml).
#[cfg(target_arch = "wasm32")]
fn load_nav2_demo(app_state: &mut AppState) {
    load_nav2_map(
        app_state,
        Nav2MapConfig {
            name: "warehouse_speed.yaml",
            image_bytes: include_bytes!("../../data/nav2_example/warehouse_speed.png"),
            yaml_bytes: include_bytes!("../../data/nav2_example/warehouse_speed.yaml"),
            pose_bytes: include_bytes!("../../data/nav2_example/map_pose_warehouse.yaml"),
            value_mode: Some(value_interpretation::Mode::Scale),
            colormap: Some(ColorMap::CoolCostmap),
            color_to_alpha: None,
        },
    );

    load_nav2_map(
        app_state,
        Nav2MapConfig {
            name: "warehouse_keepout.yaml",
            image_bytes: include_bytes!("../../data/nav2_example/warehouse_keepout.png"),
            yaml_bytes: include_bytes!("../../data/nav2_example/warehouse_keepout.yaml"),
            pose_bytes: include_bytes!("../../data/nav2_example/map_pose_warehouse.yaml"),
            value_mode: Some(value_interpretation::Mode::Trinary),
            colormap: Some(ColorMap::RvizCostmap),
            color_to_alpha: None,
        },
    );

    load_nav2_map(
        app_state,
        Nav2MapConfig {
            name: "warehouse.yaml",
            image_bytes: include_bytes!("../../data/nav2_example/warehouse.png"),
            yaml_bytes: include_bytes!("../../data/nav2_example/warehouse.yaml"),
            pose_bytes: include_bytes!("../../data/nav2_example/map_pose_warehouse.yaml"),
            value_mode: None,
            colormap: None,
            color_to_alpha: Some(egui::Color32::from_rgba_premultiplied(254, 254, 254, 255)),
        },
    );

    load_nav2_map(
        app_state,
        Nav2MapConfig {
            name: "depot_speed.yaml",
            image_bytes: include_bytes!("../../data/nav2_example/depot_speed.png"),
            yaml_bytes: include_bytes!("../../data/nav2_example/depot_speed.yaml"),
            pose_bytes: include_bytes!("../../data/nav2_example/map_pose_depot.yaml"),
            value_mode: Some(value_interpretation::Mode::Scale),
            colormap: Some(ColorMap::CoolCostmap),
            color_to_alpha: None,
        },
    );

    load_nav2_map(
        app_state,
        Nav2MapConfig {
            name: "depot_keepout.yaml",
            image_bytes: include_bytes!("../../data/nav2_example/depot_keepout.png"),
            yaml_bytes: include_bytes!("../../data/nav2_example/depot_keepout.yaml"),
            pose_bytes: include_bytes!("../../data/nav2_example/map_pose_depot.yaml"),
            value_mode: Some(value_interpretation::Mode::Trinary),
            colormap: Some(ColorMap::RvizCostmap),
            color_to_alpha: None,
        },
    );

    load_nav2_map(
        app_state,
        Nav2MapConfig {
            name: "depot.yaml",
            image_bytes: include_bytes!("../../data/nav2_example/depot.png"),
            yaml_bytes: include_bytes!("../../data/nav2_example/depot.yaml"),
            pose_bytes: include_bytes!("../../data/nav2_example/map_pose_depot.yaml"),
            value_mode: None,
            colormap: None,
            color_to_alpha: Some(egui::Color32::from_rgba_premultiplied(254, 254, 254, 255)),
        },
    );

    // Configure view settings to match the session.
    app_state.options.grid.scale = 10.0;
    app_state.options.grid.drag(egui::vec2(-30., 30.));
    app_state.options.grid.line_spacing_meters = 10.;
    app_state.options.grid.marker_visibility = MarkerVisibility::All;
    app_state.options.grid.marker_length_meters = 2.;
    app_state.options.grid.marker_width_meters = 0.3;
    app_state.options.canvas_settings.theme_preference = egui::ThemePreference::Dark;

    // Collapse some settings to be less overwhelming.
    app_state.options.collapsed.app_settings = true;
    app_state.options.collapsed.canvas_settings = true;
    app_state.options.collapsed.tool_settings = true;
}
