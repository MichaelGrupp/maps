use std::path::PathBuf;

use eframe::emath;
use serde::{Deserialize, Serialize};

use crate::movable::{Draggable, Rotatable};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MapPose {
    pub root_frame: String,
    pub map_frame: String,
    pub translation: Translation,
    pub rotation: Rotation,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Rotation {
    pub roll: f32,
    pub pitch: f32,
    pub yaw: f32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Translation {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug)]
pub struct Error {
    pub message: String,
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
    pub fn new(root_frame: String, map_frame: String) -> MapPose {
        MapPose {
            root_frame,
            map_frame,
            translation: Translation::default(),
            rotation: Rotation::default(),
        }
    }

    pub fn invert(&mut self) {
        self.translation.x = -self.translation.x;
        self.translation.y = -self.translation.y;
        self.translation.z = -self.translation.z;
        self.rotation.roll = -self.rotation.roll;
        self.rotation.pitch = -self.rotation.pitch;
        self.rotation.yaw = -self.rotation.yaw;
    }

    pub fn with_vec2(&mut self, vec: emath::Vec2) -> &mut Self {
        self.translation.x = vec.x;
        self.translation.y = vec.y;
        self
    }

    pub fn with_rot2(&mut self, rot: emath::Rot2) -> &mut Self {
        self.rotation.yaw = rot.angle();
        self
    }

    pub fn rot2(&self) -> emath::Rot2 {
        emath::Rot2::from_angle(self.rotation.yaw).normalized()
    }

    pub fn vec2(&self) -> emath::Vec2 {
        emath::vec2(self.translation.x, self.translation.y)
    }

    pub fn from_yaml_file(yaml_path: &PathBuf) -> Result<MapPose, Error> {
        match std::fs::File::open(yaml_path) {
            Ok(file) => match serde_yaml_ng::from_reader::<std::fs::File, MapPose>(file) {
                Ok(mut map_pose) => {
                    map_pose.rotation.roll = emath::normalized_angle(map_pose.rotation.roll);
                    map_pose.rotation.pitch = emath::normalized_angle(map_pose.rotation.pitch);
                    map_pose.rotation.yaw = emath::normalized_angle(map_pose.rotation.yaw);
                    Ok(map_pose)
                }
                Err(error) => Err(Error {
                    message: error.to_string(),
                }),
            },
            Err(error) => Err(Error {
                message: error.to_string(),
            }),
        }
    }

    pub fn to_yaml(&self) -> Result<String, Error> {
        match serde_yaml_ng::to_string(self) {
            Ok(yaml) => Ok(yaml),
            Err(error) => Err(Error {
                message: error.to_string(),
            }),
        }
    }

    pub fn to_yaml_file(&self, yaml_path: &PathBuf) -> Result<(), Error> {
        match std::fs::write(yaml_path, self.to_yaml()?) {
            Ok(_) => Ok(()),
            Err(error) => Err(Error {
                message: error.to_string(),
            }),
        }
    }
}
