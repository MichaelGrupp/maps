use std::path::PathBuf;

use confy;
use log::{error, info, warn};

use crate::app::AppOptions;

const APP_NAME: &str = "maps";
const APP_OPTIONS_NAME: &str = "app_options";

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
                custom_config_path: None,
                ..Default::default()
            }
        }
    }
}

pub fn save_app_options(options: &AppOptions) {
    let config_path = resolve_path_or_die(options.custom_config_path.clone());
    info!("Saving options to {:?}", config_path);
    match confy::store_path(config_path, options) {
        Ok(_) => info!("Saved options."),
        Err(e) => error!("Error saving options: {}", e),
    }
}
