use std::path::PathBuf;

use confy;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use toml;

use crate::app::{AppOptions, SessionData};
use crate::error::{Error, Result};

const APP_NAME: &str = "maps";
const APP_OPTIONS_NAME: &str = "app_options";

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

pub fn save_session(path: &PathBuf, session: &SessionData) -> Result<()> {
    info!("Saving session to {:?}", path);
    let toml = toml::to_string_pretty(session)
        .map_err(|e| Error::toml_serialize(format!("Cannot serialize session to {:?}", path), e))?;

    std::fs::write(path, toml)
        .map_err(|e| Error::io(format!("Cannot save session to {:?}", path), e))?;

    Ok(())
}

pub fn load_session(path: &PathBuf) -> Result<SessionData> {
    info!("Loading session from {:?}", path);
    let toml = std::fs::read_to_string(path)
        .map_err(|e| Error::io(format!("Cannot load session from {:?}", path), e))?;

    toml::from_str(&toml).map_err(|e| {
        Error::toml_deserialize(format!("Cannot deserialize session from {:?}", path), e)
    })
}
