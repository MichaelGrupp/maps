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
        let mut tiles = egui_tiles::Tiles::default();
        let root = tiles.insert_tab_tile(vec![]);
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
            if let Some(egui_tiles::Tile::Container(egui_tiles::Container::Tabs(tabs))) =
                self.tree.tiles.get_mut(root_id)
            {
                debug!("Adding tile {:?} for {}.", child, pane_id);
                tabs.add_child(child);
                tabs.set_active(child);
                self.tile_ids_by_pane_id.insert(pane_id, child);
            }
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
