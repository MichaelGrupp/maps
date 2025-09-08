/// VDA LIF (Layout Interchange Format) data structures.
/// Based on VDA LIF specification for robot navigation layouts.
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// Root structure of a LIF file.
#[derive(Debug, Serialize, Deserialize)]
pub struct LifFile {
    #[serde(rename = "metaInformation")]
    pub meta_information: MetaInformation,
    pub layouts: Vec<Layout>,
}

impl LifFile {
    /// Parse a LIF file from JSON string.
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(|e| Error::json("Failed to parse LIF file", e))
    }

    /// Convert LIF file to JSON string.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| Error::json("Failed to serialize LIF file", e))
    }
}

/// Metadata about the LIF file.
#[derive(Debug, Serialize, Deserialize)]
pub struct MetaInformation {
    #[serde(rename = "projectIdentification")]
    pub project_identification: String,
    pub creator: String,
    #[serde(rename = "exportTimestamp")]
    pub export_timestamp: String,
    #[serde(rename = "lifVersion")]
    pub lif_version: String,
}

/// A layout represents a complete navigation environment.
#[derive(Debug, Serialize, Deserialize)]
pub struct Layout {
    #[serde(rename = "layoutId")]
    pub layout_id: String,
    #[serde(rename = "layoutName")]
    pub layout_name: String,
    #[serde(rename = "layoutVersion")]
    pub layout_version: String,
    #[serde(rename = "layoutLevelId")]
    pub layout_level_id: Option<String>,
    #[serde(rename = "layoutDescription")]
    pub layout_description: Option<String>,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub stations: Option<Vec<Station>>,
}

/// A node represents a point in the navigation graph.
#[derive(Debug, Serialize, Deserialize)]
pub struct Node {
    #[serde(rename = "nodeId")]
    pub node_id: String,
    #[serde(rename = "nodeName")]
    pub node_name: Option<String>,
    #[serde(rename = "nodeDescription")]
    pub node_description: Option<String>,
    #[serde(rename = "mapId")]
    pub map_id: String,
    #[serde(rename = "nodePosition")]
    pub node_position: Position,
    #[serde(rename = "vehicleTypeNodeProperties")]
    pub vehicle_type_node_properties: Vec<VehicleTypeNodeProperty>,
}

/// Position coordinates.
#[derive(Debug, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

/// Station position coordinates with orientation.
#[derive(Debug, Serialize, Deserialize)]
pub struct StationPosition {
    pub x: f64,
    pub y: f64,
    pub theta: f64,
}

/// Control point for trajectory with optional weight.
#[derive(Debug, Serialize, Deserialize)]
pub struct ControlPoint {
    pub x: f64,
    pub y: f64,
    #[serde(default = "default_weight")]
    pub weight: f64,
}

/// Default weight value for control points (1.0 as per LIF specification).
fn default_weight() -> f64 {
    1.0
}

/// Vehicle type specific properties for a node.
#[derive(Debug, Serialize, Deserialize)]
pub struct VehicleTypeNodeProperty {
    #[serde(rename = "vehicleTypeId")]
    pub vehicle_type_id: String,
    pub theta: Option<f64>,
    pub actions: Option<Vec<Action>>,
}

/// An edge represents a connection between two nodes.
#[derive(Debug, Serialize, Deserialize)]
pub struct Edge {
    #[serde(rename = "edgeId")]
    pub edge_id: String,
    #[serde(rename = "edgeName")]
    pub edge_name: Option<String>,
    #[serde(rename = "edgeDescription")]
    pub edge_description: Option<String>,
    #[serde(rename = "startNodeId")]
    pub start_node_id: String,
    #[serde(rename = "endNodeId")]
    pub end_node_id: String,
    #[serde(rename = "vehicleTypeEdgeProperties")]
    pub vehicle_type_edge_properties: Vec<VehicleTypeEdgeProperty>,
}

/// Vehicle type specific properties for an edge.
#[derive(Debug, Serialize, Deserialize)]
pub struct VehicleTypeEdgeProperty {
    #[serde(rename = "vehicleTypeId")]
    pub vehicle_type_id: String,
    #[serde(rename = "vehicleOrientation")]
    pub vehicle_orientation: Option<f64>,
    #[serde(rename = "orientationType")]
    pub orientation_type: String,
    #[serde(rename = "rotationAllowed")]
    pub rotation_allowed: bool,
    #[serde(rename = "rotationAtStartNodeAllowed")]
    pub rotation_at_start_node_allowed: Option<String>,
    #[serde(rename = "rotationAtEndNodeAllowed")]
    pub rotation_at_end_node_allowed: Option<String>,
    #[serde(rename = "maxSpeed")]
    pub max_speed: Option<f64>,
    #[serde(rename = "maxRotationSpeed")]
    pub max_rotation_speed: Option<f64>,
    #[serde(rename = "minHeight")]
    pub min_height: Option<f64>,
    #[serde(rename = "maxHeight")]
    pub max_height: Option<f64>,
    #[serde(rename = "loadRestriction")]
    pub load_restriction: Option<LoadRestriction>,
    pub trajectory: Option<Trajectory>,
    pub actions: Option<Vec<Action>>,
    #[serde(rename = "reentryAllowed")]
    pub reentry_allowed: Option<bool>,
}

/// Trajectory definition for curved paths.
#[derive(Debug, Serialize, Deserialize)]
pub struct Trajectory {
    pub degree: u32,
    #[serde(rename = "knotVector")]
    pub knot_vector: Vec<f64>,
    #[serde(rename = "controlPoints")]
    pub control_points: Vec<ControlPoint>,
}

/// An action that can be performed by a vehicle.
#[derive(Debug, Serialize, Deserialize)]
pub struct Action {
    #[serde(rename = "actionType")]
    pub action_type: String,
    #[serde(rename = "actionDescription")]
    pub action_description: Option<String>,
    #[serde(rename = "requirementType")]
    pub requirement_type: String,
    #[serde(rename = "blockingType")]
    pub blocking_type: String,
    #[serde(rename = "actionParameters")]
    pub action_parameters: Option<Vec<ActionParameter>>,
}

/// Action parameter with key-value pairs.
#[derive(Debug, Serialize, Deserialize)]
pub struct ActionParameter {
    pub key: String,
    pub value: String,
}

/// Load restriction for vehicle edges.
#[derive(Debug, Serialize, Deserialize)]
pub struct LoadRestriction {
    pub unloaded: Option<bool>,
    pub loaded: Option<bool>,
    #[serde(rename = "loadSetNames")]
    pub load_set_names: Option<Vec<String>>,
}

/// A station represents an interaction point.
#[derive(Debug, Serialize, Deserialize)]
pub struct Station {
    #[serde(rename = "stationId")]
    pub station_id: String,
    #[serde(rename = "interactionNodeIds")]
    pub interaction_node_ids: Vec<String>,
    #[serde(rename = "stationName")]
    pub station_name: Option<String>,
    #[serde(rename = "stationDescription")]
    pub station_description: Option<String>,
    #[serde(rename = "stationHeight")]
    pub station_height: Option<f64>,
    #[serde(rename = "stationPosition")]
    pub station_position: Option<StationPosition>,
}
