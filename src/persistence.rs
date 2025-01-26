use confy;
use log::{error, info, warn};

use crate::app::AppOptions;

const APP_NAME: &str = "maps";
const APP_OPTIONS_NAME: &str = "app_options";

pub fn load_app_options() -> AppOptions {
    info!(
        "Loading options from {:?}",
        confy::get_configuration_file_path(APP_NAME, Some(APP_OPTIONS_NAME)).unwrap_or_default()
    );
    match confy::load(APP_NAME, Some(APP_OPTIONS_NAME)) {
        Ok(options) => options,
        Err(e) => {
            warn!(
                "Error loading options from {:?}: {}. Using defaults.",
                confy::get_configuration_file_path(APP_NAME, Some(APP_OPTIONS_NAME))
                    .unwrap_or_default(),
                e
            );
            AppOptions::default()
        }
    }
}

pub fn save_app_options(options: &AppOptions) {
    info!(
        "Saving options to {:?}",
        confy::get_configuration_file_path(APP_NAME, Some(APP_OPTIONS_NAME)).unwrap_or_default()
    );
    match confy::store(APP_NAME, Some(APP_OPTIONS_NAME), options) {
        Ok(_) => info!("Saved options."),
        Err(e) => error!("Error saving options: {}", e),
    }
}
