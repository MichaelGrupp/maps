/// Internal layout representation.
use std::collections::HashMap;

use eframe::egui;
use serde::{Deserialize, Serialize};

use crate::graph::vda_lif;

/// A simplified layout for visualization purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layout {
    pub layout_id: String,
    pub layout_name: String,
    pub nodes: HashMap<String, Node>,
    pub edges: Vec<Edge>,
    pub stations: Vec<Station>,
}

/// A node in the visualization graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub node_id: String,
    pub position: egui::Pos2,
    /// Vehicle types that can use this node.
    pub vehicle_types: Vec<String>,
}

/// An edge connecting two nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub edge_id: String,
    pub start_node_id: String,
    pub end_node_id: String,
}

/// A station representing an interaction point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Station {
    pub station_id: String,
    pub station_name: String,
    pub position: Option<egui::Pos2>,
    /// Node IDs where this station can interact.
    pub interaction_node_ids: Vec<String>,
}

impl Layout {
    /// Convert from VDA LIF layout to internal visualization layout.
    pub fn from_vda_lif(lif_layout: &vda_lif::Layout) -> Self {
        let mut nodes = HashMap::new();
        let mut edges = Vec::new();
        let mut stations = Vec::new();

        // Convert nodes.
        for lif_node in &lif_layout.nodes {
            let node = Node {
                node_id: lif_node.node_id.clone(),
                position: egui::pos2(
                    lif_node.node_position.x as f32,
                    lif_node.node_position.y as f32,
                ),
                vehicle_types: lif_node
                    .vehicle_type_node_properties
                    .iter()
                    .map(|prop| prop.vehicle_type_id.clone())
                    .collect(),
            };
            nodes.insert(node.node_id.clone(), node);
        }

        // Convert edges.
        for lif_edge in &lif_layout.edges {
            let edge = Edge {
                edge_id: lif_edge.edge_id.clone(),
                start_node_id: lif_edge.start_node_id.clone(),
                end_node_id: lif_edge.end_node_id.clone(),
            };
            edges.push(edge);
        }

        // Convert stations.
        if let Some(lif_stations) = &lif_layout.stations {
            for lif_station in lif_stations {
                let station = Station {
                    station_id: lif_station.station_id.clone(),
                    station_name: lif_station.station_name.clone(),
                    position: lif_station
                        .station_position
                        .as_ref()
                        .map(|pos| egui::pos2(pos.x as f32, pos.y as f32)),
                    interaction_node_ids: lif_station.interaction_node_ids.clone(),
                };
                stations.push(station);
            }
        }

        Self {
            layout_id: lif_layout.layout_id.clone(),
            layout_name: lif_layout.layout_name.clone(),
            nodes,
            edges,
            stations,
        }
    }
}
