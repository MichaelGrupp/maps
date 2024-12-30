use log::debug;
use std::collections::HashMap;

use eframe::egui;
use egui_tiles;

use crate::map_state::MapState;
use crate::texture_request::TextureRequest;
use crate::tiles::Pane;

// Behavior for the tiles tree that displays maps.
pub struct MapsTreeBehavior<'a> {
    pub maps: &'a mut HashMap<String, MapState>,
}

impl<'a> egui_tiles::Behavior<Pane> for MapsTreeBehavior<'a> {
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
            egui::ScrollArea::both().show(ui, |ui| {
                map.texture_state
                    .update(ui, &TextureRequest::new(pane.id.clone(), ui.clip_rect()));
                let texture = match &map.texture_state.texture_handle {
                    Some(texture) => texture,
                    None => {
                        panic!("Missing texture handle for image {}", pane.id);
                    }
                };
                let image = match map.tint {
                    Some(tint) => egui::Image::new(texture).tint(tint),
                    None => egui::Image::new(texture),
                };
                let image_response = ui.add(image).interact(egui::Sense::click_and_drag());
                if image_response.drag_started_by(egui::PointerButton::Primary) {
                    debug!("Dragging image {}", pane.id);
                    tiles_response = egui_tiles::UiResponse::DragStarted;
                }
                map.texture_state.image_response = Some(image_response);
            });
        }
        tiles_response
    }
}
