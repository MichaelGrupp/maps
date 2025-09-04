//! Map metadata.

use eframe::emath;
use log::debug;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::path_helpers::resolve_symlink;
use crate::value_interpretation::{Mode, ValueInterpretation};

/// Plain ROS map metadata yaml file format.
#[derive(Deserialize)]
pub struct MetaYaml {
    pub image: PathBuf,
    pub resolution: f32,
    pub origin: [f32; 3], // x, y, theta
    pub negate: i32,
    pub occupied_thresh: f32,
    pub free_thresh: f32,
    pub mode: Option<Mode>,
}

// Annotated yaml meta to keep track of the yaml file path.
struct MetaYamlAnnotated {
    pub meta_yaml: MetaYaml,
    pub yaml_path: PathBuf,
}

impl MetaYamlAnnotated {
    fn from(yaml_path: &Path) -> Result<MetaYamlAnnotated> {
        let yaml_path = resolve_symlink(yaml_path);
        let buffer = std::fs::read_to_string(&yaml_path)
            .map_err(|e| Error::io(format!("Cannot read {yaml_path:?}"), e))?;

        let meta_yaml = serde_yaml_ng::from_str::<MetaYaml>(&buffer)
            .map_err(|e| Error::yaml(format!("Cannot parse {yaml_path:?}"), e))?;

        Ok(MetaYamlAnnotated {
            meta_yaml,
            yaml_path,
        })
    }

    fn from_bytes(bytes: &[u8], yaml_name: &str) -> Result<MetaYamlAnnotated> {
        let meta_yaml = serde_yaml_ng::from_slice::<MetaYaml>(bytes)
            .map_err(|e| Error::yaml(format!("Cannot parse {yaml_name}"), e))?;

        Ok(MetaYamlAnnotated {
            yaml_path: PathBuf::from(yaml_name),
            meta_yaml,
        })
    }
}

/// Internal representation of the metadata.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Meta {
    pub image_path: PathBuf,
    pub yaml_path: PathBuf,
    pub resolution: f32,
    pub origin_xy: emath::Vec2,
    pub origin_theta: emath::Rot2,
    #[serde(default)]
    pub value_interpretation: ValueInterpretation,
    #[serde(default)]
    original_value_interpretation: ValueInterpretation,
}

impl From<MetaYamlAnnotated> for Meta {
    fn from(meta_yaml_annotated: MetaYamlAnnotated) -> Meta {
        let meta_yaml = &meta_yaml_annotated.meta_yaml;
        Meta {
            // Resolve image path, it can be absolute or relative to the yaml file.
            image_path: if meta_yaml.image.is_absolute() {
                resolve_symlink(&meta_yaml.image)
            } else {
                meta_yaml_annotated
                    .yaml_path
                    .parent()
                    .expect("YAML file has no parent directory")
                    .join(&meta_yaml.image)
            },
            yaml_path: meta_yaml_annotated.yaml_path,
            resolution: meta_yaml.resolution,
            origin_xy: emath::Vec2::new(meta_yaml.origin[0], meta_yaml.origin[1]),
            origin_theta: emath::Rot2::from_angle(emath::normalized_angle(meta_yaml.origin[2])),
            value_interpretation: ValueInterpretation::from_meta_yaml(meta_yaml),
            original_value_interpretation: ValueInterpretation::from_meta_yaml(meta_yaml),
        }
    }
}

impl Meta {
    pub fn load_from_file(yaml_path: &Path) -> Result<Meta> {
        let meta_yaml_annotated = MetaYamlAnnotated::from(yaml_path)?;
        let meta = Meta::from(meta_yaml_annotated);
        debug!("Parsed metadata: {:?}", meta);
        if !meta.image_path.exists() {
            return Err(Error::app(format!(
                "Metadata from {} points to an image that does not exist: {}",
                yaml_path.display(),
                meta.image_path.display()
            )));
        }
        Ok(meta)
    }

    pub fn load_from_bytes(bytes: &[u8], yaml_name: &str) -> Result<Meta> {
        let meta_yaml_annotated = MetaYamlAnnotated::from_bytes(bytes, yaml_name)?;
        let meta = Meta::from(meta_yaml_annotated);
        debug!("Parsed metadata from bytes: {:?}", meta);
        Ok(meta)
    }

    pub fn reset_value_interpretation(&mut self) {
        self.value_interpretation = self.original_value_interpretation;
    }
}
