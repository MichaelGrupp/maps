use std::{collections::BTreeMap, path::PathBuf};

use confy;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use toml;

use crate::app::AppOptions;
use crate::map_state::MapState;

const APP_NAME: &str = "maps";
const APP_OPTIONS_NAME: &str = "app_options";

#[derive(Debug)]
pub struct Error {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PersistenceOptions {
    pub custom_config_path: Option<PathBuf>,
    pub autosave: bool,
}

impl Default for PersistenceOptions {
    fn default() -> Self {
        PersistenceOptions {
            custom_config_path: None,
            autosave: true,
        }
    }
}

fn resolve_path_or_die(custom_path: Option<PathBuf>) -> PathBuf {
    custom_path.unwrap_or_else(|| {
        confy::get_configuration_file_path(APP_NAME, Some(APP_OPTIONS_NAME))
            .expect("Fatal: failed to resolve any configuration path.")
    })
}

pub fn load_app_options(custom_path: &Option<PathBuf>) -> AppOptions {
    let config_path = resolve_path_or_die(custom_path.clone());
    info!("Loading options from {:?}", config_path);
    match confy::load_path(config_path.as_path()) {
        Ok(options) => options,
        Err(e) => {
            warn!(
                "Error loading options from {:?}: {}. Using defaults.",
                config_path, e
            );
            // Don't use the custom path here, it might be from a different version
            // or an user typo pointing to some random file. So we shouldn't save to it later.
            AppOptions {
                persistence: PersistenceOptions {
                    custom_config_path: None,
                    ..Default::default()
                },
                ..Default::default()
            }
        }
    }
}

pub fn save_app_options(options: &AppOptions) {
    let config_path = resolve_path_or_die(options.persistence.custom_config_path.clone());
    info!("Saving options to {:?}", config_path);
    match confy::store_path(config_path, options) {
        Ok(_) => (),
        Err(e) => error!("Error saving options: {}", e),
    }
}

pub fn save_map_states(path: &PathBuf, maps: &BTreeMap<String, MapState>) -> Result<(), Error> {
    match toml::to_string(maps) {
        Ok(toml) => {
            info!("Saving map states to {:?}", path);
            match std::fs::write(path, toml) {
                Ok(_) => (),
                Err(e) => {
                    return Err(Error {
                        message: format!("Error saving map state: {}", e),
                    });
                }
            }
        }
        Err(e) => {
            return Err(Error {
                message: format!("Error serializing map state: {}", e),
            });
        }
    }
    Ok(())
}

pub fn load_map_states(path: &PathBuf) -> Result<BTreeMap<String, MapState>, Error> {
    info!("Loading map states from {:?}", path);
    match std::fs::read_to_string(path) {
        Ok(toml) => match toml::from_str(&toml) {
            Ok(maps) => Ok(maps),
            Err(e) => Err(Error {
                message: format!("Error deserializing map state: {}", e),
            }),
        },
        Err(e) => Err(Error {
            message: format!("Error loading map state: {}", e),
        }),
    }
}
