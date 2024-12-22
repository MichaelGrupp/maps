use nalgebra::Isometry2;
use serde::Deserialize;
use std::path::PathBuf;

// Plain ROS map metadata yaml file format.
#[derive(Deserialize)]
pub struct MetaYaml {
    pub image: PathBuf,
    pub resolution: f64,
    pub origin: [f64; 3], // x, y, theta
    pub negate: i32,
    pub occupied_thresh: f64,
    pub free_thresh: f64,
}

// Annotated yaml meta to keep track of the yaml file path.
pub struct MetaYamlAnnotated {
    pub meta_yaml: MetaYaml,
    pub yaml_path: PathBuf,
}

#[derive(Debug)]
pub struct Error {
    pub message: String,
}

impl MetaYamlAnnotated {
    pub fn from(yaml_path: PathBuf) -> Result<MetaYamlAnnotated, Error> {
        match std::fs::read_to_string(&yaml_path) {
            Ok(buffer) => match serde_yml::from_str::<MetaYaml>(&buffer) {
                Ok(meta_yaml) => Ok(MetaYamlAnnotated {
                    meta_yaml,
                    yaml_path,
                }),
                Err(e) => Err(Error {
                    message: format!("Failed to parse yaml: {}", e),
                }),
            },
            Err(e) => Err(Error {
                message: format!("Failed to read yaml file: {}", e),
            }),
        }
    }
}

// Internal representation of the metadata.
#[derive(Debug)]
pub struct Meta {
    pub image_path: PathBuf,
    pub yaml_path: PathBuf,
    pub resolution: f64,
    pub origin: Isometry2<f64>,
    // negate, occupied_thresh, free_thresh are not used
}

impl From<MetaYamlAnnotated> for Meta {
    fn from(meta_yaml_annotated: MetaYamlAnnotated) -> Meta {
        let meta_yaml = &meta_yaml_annotated.meta_yaml;
        Meta {
            // Resolve image path, it can be absolute or relative to the yaml file.
            image_path: if meta_yaml.image.is_absolute() {
                meta_yaml.image.clone()
            } else {
                meta_yaml_annotated
                    .yaml_path
                    .parent()
                    .unwrap()
                    .join(&meta_yaml.image)
            },
            yaml_path: meta_yaml_annotated.yaml_path,
            resolution: meta_yaml.resolution,
            origin: Isometry2::new(
                nalgebra::Vector2::new(meta_yaml.origin[0], meta_yaml.origin[1]),
                meta_yaml.origin[2],
            ),
        }
    }
}

impl Meta {
    pub fn load_from_file(yaml_path: PathBuf) -> Result<Meta, Error> {
        match MetaYamlAnnotated::from(yaml_path) {
            Ok(meta_yaml_annotated) => Ok(Meta::from(meta_yaml_annotated)),
            Err(e) => Err(e),
        }
    }
}
