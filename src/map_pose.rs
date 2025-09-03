//! Pose utilities for map alignment.

use std::path::PathBuf;

use eframe::emath;
use log::debug;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::movable::{Draggable, Rotatable};
use crate::path_helpers::resolve_symlink;

/// Pose of a map in metric coordinates.
/// Allows to align the map independently of the map metadata file contents.
/// Can be used for external applications through saving to a YAML file.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MapPose {
    pub translation: Translation,
    pub rotation: Rotation,

    /// The name of the coordinate frame that the map pose is relative to.
    /// Can be left empty if it's not needed for your use case.
    #[serde(default)]
    pub root_frame: String,
    /// The name of the coordinate frame of the map itself.
    /// Can be left empty if it's not needed for your use case.
    #[serde(default)]
    pub map_frame: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Rotation {
    /// Placeholder for roll. Not used by the maps app.
    #[serde(default)]
    pub roll: f32,
    /// Placeholder for pitch. Not used by the maps app.
    #[serde(default)]
    pub pitch: f32,

    /// Yaw angle in radians.
    pub yaw: f32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Translation {
    pub x: f32,
    pub y: f32,

    /// Placeholder for z. Not used by the maps app.
    #[serde(default)]
    pub z: f32,
}

impl Draggable for MapPose {
    fn offset_rhs(&self) -> emath::Vec2 {
        emath::vec2(self.translation.x, self.translation.y)
    }

    fn drag(&mut self, delta: emath::Vec2) {
        self.translation.x += delta.x;
        self.translation.y -= delta.y;
    }
}

impl Rotatable for MapPose {
    fn rotate(&mut self, delta: f32) {
        self.rotation.yaw = emath::normalized_angle(self.rotation.yaw + delta);
    }
}

impl MapPose {
    /// Creates a new map pose with frame metadata and default pose values.
    pub fn new(root_frame: String, map_frame: String) -> MapPose {
        MapPose {
            root_frame,
            map_frame,
            translation: Translation::default(),
            rotation: Rotation::default(),
        }
    }

    /// In-place inversion of the pose.
    pub fn invert(&mut self) {
        self.translation.x = -self.translation.x;
        self.translation.y = -self.translation.y;
        self.translation.z = -self.translation.z;
        self.rotation.roll = -self.rotation.roll;
        self.rotation.pitch = -self.rotation.pitch;
        self.rotation.yaw = -self.rotation.yaw;
    }

    /// Builder pattern for setting the translation.
    pub fn with_vec2(&mut self, vec: emath::Vec2) -> &mut Self {
        self.translation.x = vec.x;
        self.translation.y = vec.y;
        self
    }

    /// Builder pattern for setting the rotation.
    pub fn with_rot2(&mut self, rot: emath::Rot2) -> &mut Self {
        self.rotation.yaw = rot.angle();
        self
    }

    /// Converts the rotation to an `emath` type.
    pub fn rot2(&self) -> emath::Rot2 {
        emath::Rot2::from_angle(self.rotation.yaw).normalized()
    }

    /// Converts the translation to an `emath` type.
    pub fn vec2(&self) -> emath::Vec2 {
        emath::vec2(self.translation.x, self.translation.y)
    }

    fn normalized(mut self) -> MapPose {
        self.rotation.roll = emath::normalized_angle(self.rotation.roll);
        self.rotation.pitch = emath::normalized_angle(self.rotation.pitch);
        self.rotation.yaw = emath::normalized_angle(self.rotation.yaw);
        self
    }

    /// Loads a map pose from a YAML file.
    /// Note that angles are normalized to the range [-π, π] by this.
    pub fn from_yaml_file(yaml_path: &PathBuf) -> Result<MapPose> {
        let file = std::fs::File::open(resolve_symlink(yaml_path))
            .map_err(|e| Error::io(format!("Cannot open map pose file {:?}", yaml_path), e))?;

        let map_pose = serde_yaml_ng::from_reader::<std::fs::File, MapPose>(file)
            .map_err(|e| Error::yaml(format!("Cannot parse map pose from {:?}", yaml_path), e))?
            .normalized();

        debug!(
            "Loaded and normalized map pose from {:?}: {:?}",
            yaml_path, map_pose
        );
        Ok(map_pose)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn from_bytes(bytes: &[u8]) -> Result<MapPose> {
        let map_pose = serde_yaml_ng::from_slice::<MapPose>(bytes)
            .map_err(|e| Error::yaml("Cannot parse map pose from bytes", e))?
            .normalized();
        debug!("Loaded and normalized map pose from bytes: {:?}", map_pose);
        Ok(map_pose)
    }

    /// Serializes the map pose to a YAML string.
    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml_ng::to_string(self)
            .map_err(|e| Error::yaml("Cannot serialize map pose to YAML", e))
    }

    pub fn to_yaml_file(&self, yaml_path: &PathBuf) -> Result<()> {
        let yaml_content = self.to_yaml()?;
        std::fs::write(yaml_path, yaml_content)
            .map_err(|e| Error::io(format!("Cannot write map pose to {:?}", yaml_path), e))
    }

    #[cfg(target_arch = "wasm32")]
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let yaml = self.to_yaml()?;
        Ok(yaml.into_bytes())
    }
}
