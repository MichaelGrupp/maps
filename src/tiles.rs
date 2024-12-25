use core::panic;
use std::collections::HashMap;
use std::default;

use egui_tiles;
use log::debug;
use uuid::Uuid;

pub struct Pane {
    pub id: String,
}

// Simplifies managing a tiles tree for panes that hold an user content ID.
pub struct Tiles {
    pub tree: egui_tiles::Tree<Pane>,
    tile_ids_by_pane_id: HashMap<String, egui_tiles::TileId>,
}

impl default::Default for Tiles {
    fn default() -> Tiles {
        // We start with a single tab tile.
        // An invisible root pane is inserted to prevent the root tile from being removed.
        // See: https://github.com/rerun-io/egui_tiles/issues/83
        let mut tiles = egui_tiles::Tiles::default();
        let root_pane_id = tiles.insert_pane(Pane {
            id: "_root_pane".to_string(),
        });
        tiles.set_visible(root_pane_id, false);
        let root = tiles.insert_tab_tile(vec![root_pane_id]);
        Tiles {
            // ID must be globally unique, see Tree:new().
            tree: egui_tiles::Tree::new(Uuid::new_v4().to_string(), root, tiles),
            tile_ids_by_pane_id: HashMap::new(),
        }
    }
}

impl Tiles {
    pub fn add_pane(&mut self, pane: Pane) {
        if self.tile_ids_by_pane_id.contains_key(&pane.id) {
            return;
        }
        let pane_id = pane.id.clone();
        let child = self.tree.tiles.insert_pane(pane);
        if let Some(root_id) = self.tree.root() {
            // The root tile is not necessarily a tab tile anymore if the user rearranged the layout,
            // but should be still a container.
            if let Some(egui_tiles::Tile::Container(root_container)) =
                self.tree.tiles.get_mut(root_id)
            {
                debug!(
                    "Adding tile {:?} for {} to root container ({:?}).",
                    child,
                    pane_id,
                    root_container.kind()
                );
                root_container.add_child(child);
                self.tile_ids_by_pane_id.insert(pane_id, child);
            } else {
                panic!("Root tile is not a container.");
            }
        } else {
            panic!("Root tile not found.");
        }
    }

    pub fn remove_pane(&mut self, pane_id: &str) {
        if let Some(tile_id) = self.tile_ids_by_pane_id.get(pane_id) {
            debug!("Removing tile {:?} of {}", tile_id, pane_id);
            self.tree.remove_recursively(*tile_id);
            self.tile_ids_by_pane_id.remove(pane_id);
        }
    }
}
