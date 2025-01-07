use std::path::PathBuf;

use eframe::emath;
use serde::{Deserialize, Serialize};

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

impl MapPose {
    pub fn new(root_frame: String, map_frame: String) -> MapPose {
        MapPose {
            root_frame,
            map_frame,
            translation: Translation::default(),
            rotation: Rotation::default(),
        }
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
        emath::Rot2::from_angle(self.rotation.yaw)
    }

    pub fn vec2(&self) -> emath::Vec2 {
        emath::vec2(self.translation.x, self.translation.y)
    }

    pub fn from_yaml_file(yaml_path: PathBuf) -> Result<MapPose, Error> {
        match std::fs::File::open(yaml_path) {
            Ok(file) => match serde_yml::from_reader(file) {
                Ok(map_pose) => Ok(map_pose),
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
        match serde_yml::to_string(self) {
            Ok(yaml) => Ok(yaml),
            Err(error) => Err(Error {
                message: error.to_string(),
            }),
        }
    }

    pub fn to_yaml_file(&self, yaml_path: PathBuf) -> Result<(), Error> {
        match std::fs::write(yaml_path, self.to_yaml()?) {
            Ok(_) => Ok(()),
            Err(error) => Err(Error {
                message: error.to_string(),
            }),
        }
    }
}
