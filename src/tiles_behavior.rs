use log::debug;
use std::collections::BTreeMap;

use eframe::egui;

use crate::map_state::MapState;
use crate::texture_request::TextureRequest;
use crate::tiles::Pane;

// Behavior for the tiles tree that displays maps.
pub struct MapsTreeBehavior<'a> {
    pub maps: &'a mut BTreeMap<String, MapState>,
    pub hovered_id: Option<String>,
}

impl egui_tiles::Behavior<Pane> for MapsTreeBehavior<'_> {
    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        pane.id.clone().into()
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        pane: &mut Pane,
    ) -> egui_tiles::UiResponse {
        let mut tiles_response = egui_tiles::UiResponse::None;
        if let Some(map) = self.maps.get_mut(&pane.id) {
            let request = TextureRequest::new(pane.id.clone(), ui.clip_rect())
                .with_color_to_alpha(map.color_to_alpha)
                .with_tint(map.tint)
                .with_thresholding(map.get_value_interpretation())
                .with_texture_options(&map.texture_filter.get())
                .with_sense(egui::Sense::click_and_drag());

            egui::ScrollArea::both().show(ui, |ui| {
                let texture_state = map.get_or_create_texture_state(pane.id.as_str());

                texture_state.update(ui, &request);

                texture_state.put(ui, &request);
                if let Some(image_response) = &texture_state.image_response {
                    if image_response.drag_started_by(egui::PointerButton::Primary) {
                        debug!("Dragging image {}", pane.id);
                        tiles_response = egui_tiles::UiResponse::DragStarted;
                    } else if image_response.hovered() {
                        self.hovered_id = Some(pane.id.clone());
                    }
                }
            });
        }
        tiles_response
    }
}
