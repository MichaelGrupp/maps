use std::sync::Arc;

use eframe::egui;
use image::imageops::FilterType;

use crate::app::AppState;
use crate::grid_options::MarkerVisibility;
use crate::image::{load_image_from_bytes, to_egui_image};
use crate::image_pyramid::ImagePyramid;
use crate::map_pose::MapPose;
use crate::meta::Meta;
use crate::movable::Draggable;
use crate::value_interpretation::Mode;

// Embedded demo function with dummy maps (included during compile time) for the wasm target.
// This is a simple way to provide a demo without the need for a server.
#[cfg(target_arch = "wasm32")]
impl AppState {
    pub(crate) fn load_demo_maps_button(&mut self, ui: &mut egui::Ui) {
        let demo_img_texture = ui.ctx().load_texture(
            "_demo_button",
            to_egui_image(
                load_image_from_bytes(include_bytes!("../../data/doc/demo.png"))
                    .expect("broken demo")
                    .resize(350, 300, FilterType::Lanczos3),
            ),
            Default::default(),
        );
        if ui.add(egui::ImageButton::new(&demo_img_texture)).clicked() {
            self.load_dummy_maps();
        }
    }

    fn load_dummy_maps(&mut self) {
        let name_1 = "dummy_map_hires.yaml".to_string();
        let img_1 = Arc::new(ImagePyramid::new(
            load_image_from_bytes(include_bytes!("../../data/dummy_maps/dummy_map_hires.png"))
                .expect("broken demo"),
        ));
        let meta_1 = Meta::load_from_bytes(
            include_bytes!("../../data/dummy_maps/dummy_map_hires.yaml"),
            &name_1,
        )
        .expect("broken demo");
        self.add_map(&name_1, meta_1, img_1);

        let name_2 = "dummy_map_lores.yaml".to_string();
        let img_2 = Arc::new(ImagePyramid::new(
            load_image_from_bytes(include_bytes!("../../data/dummy_maps/dummy_map_lores.png"))
                .expect("broken demo"),
        ));
        let meta_2 = Meta::load_from_bytes(
            include_bytes!("../../data/dummy_maps/dummy_map_lores.yaml"),
            &name_2,
        )
        .expect("broken demo");
        self.add_map(&name_2, meta_2, img_2);

        // Change some random stuff to make it more interesting.
        let map_1 = self.data.maps.get_mut(&name_1).expect("missing demo map");
        map_1.use_value_interpretation = true;
        map_1.meta.value_interpretation.mode = Mode::Trinary;
        map_1.tint = Some(egui::Color32::from_white_alpha(128));
        map_1.pose = MapPose::from_bytes(include_bytes!(
            "../../data/google_cartographer_example/pose_1.yaml"
        ))
        .expect("broken demo");

        let map_2 = self.data.maps.get_mut(&name_2).expect("missing demo map");
        map_2.tint = Some(egui::Color32::from_white_alpha(128));
        map_2.pose.map_frame = "map_2".to_string();
        map_2.pose.root_frame = "root".to_string();

        self.options.tint_settings.active_tint_selection = Some(name_1.clone());
        self.options.grid.scale = 4.5;
        self.options.grid.drag(egui::vec2(0., 30.));
        self.options.grid.line_spacing_meters = 25.;
        self.options.grid.marker_visibility = MarkerVisibility::All;
        self.options.grid.marker_length_meters = 5.;
        self.options.grid.marker_width_meters = 1.;
    }
}
