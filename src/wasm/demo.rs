use std::sync::Arc;

use eframe::egui;

use crate::app::AppState;
use crate::grid_options::MarkerVisibility;
use crate::image::{load_image_from_bytes, to_egui_image};
use crate::image_pyramid::ImagePyramid;
use crate::map_pose::MapPose;
use crate::meta::Meta;
use crate::movable::Draggable;

// Embedded demo function with dummy maps (included during compile time) for the wasm target.
// This is a simple way to provide a demo without the need for a server.
#[cfg(target_arch = "wasm32")]
impl AppState {
    pub(crate) fn load_demo_maps_button(&mut self, ui: &mut egui::Ui) {
        if self.data.demo_button_image_handle.is_none() {
            self.data.demo_button_image_handle = Some(
                ui.ctx().load_texture(
                    "_demo_button",
                    to_egui_image(
                        load_image_from_bytes(include_bytes!("../../data/doc/demo_scaled.png"))
                            .expect("broken demo"),
                    ),
                    Default::default(),
                ),
            );
        }
        if ui
            .add(
                egui::ImageButton::new(
                    egui::Image::new(
                        self.data
                            .demo_button_image_handle
                            .as_ref()
                            .expect("missing demo image"),
                    )
                    .max_height(200.),
                )
                .corner_radius(5.),
            )
            .clicked()
        {
            self.load_demo_maps();
        }
    }

    fn load_demo_maps(&mut self) {
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
                self.add_map(&name, meta, img);

                let map = self.data.maps.get_mut(&name).expect("missing demo map");
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

        // Load maps 0, 1, and 2
        load_map!(0);
        load_map!(1);
        load_map!(2);

        self.options.tint_settings.active_tint_selection = Some("map_2.yaml".to_owned());
        self.options.grid.scale = 5.5;
        self.options.grid.drag(egui::vec2(-75., 30.));
        self.options.grid.line_spacing_meters = 25.;
        self.options.grid.marker_visibility = MarkerVisibility::All;
        self.options.grid.marker_length_meters = 5.;
        self.options.grid.marker_width_meters = 0.75;
        self.options.canvas_settings.theme_preference = egui::ThemePreference::Dark;

        // Collapse some settings to be less overwhelming and to show that it's possible.
        // (show grid settings expanded)
        self.options.collapsed.app_settings = true;
        self.options.collapsed.canvas_settings = true;
        self.options.collapsed.tool_settings = true;
    }
}
