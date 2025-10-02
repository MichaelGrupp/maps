//! Error handling for the `maps` crate.

use log::error;
use thiserror::Error;

use maps_io_ros::impl_error_constructors;

pub type Result<T> = std::result::Result<T, Error>;

/// Error types for the `maps` crate.
/// Allows to handle app errors and wrapped external errors in a unified way.
#[derive(Error, Debug)]
pub enum Error {
    /// A `maps`-internal and application-specific error.
    #[error("{message}")]
    App { message: String },

    /// Error from maps_io_ros (I/O, YAML parsing, etc.)
    #[error(transparent)]
    Core(#[from] maps_io_ros::Error),

    /// Error from maps_rendering (image processing, texture management, etc.)
    #[error(transparent)]
    Rendering(#[from] maps_rendering::error::Error),

    /// TOML deserialization error with additional context.
    #[error("[TOML error] {context} ({source})")]
    TomlDeserialize {
        context: String,
        #[source]
        source: toml::de::Error,
    },

    /// TOML serialization error with additional context.
    #[error("[TOML error] {context} ({source})")]
    TomlSerialize {
        context: String,
        #[source]
        source: toml::ser::Error,
    },
}

impl Error {
    /// Create a new app-related error with a full error message.
    #[allow(clippy::needless_pass_by_value)]
    pub fn app(message: impl ToString) -> Self {
        Self::App {
            message: message.to_string(),
        }
    }

    // Generate the wrapping error constructors.
    impl_error_constructors! {
        toml_deserialize => TomlDeserialize, toml::de::Error;
        toml_serialize => TomlSerialize, toml::ser::Error;
    }

    /// Create I/O errors by delegating to maps_io_ros
    pub fn io(context: impl ToString, source: std::io::Error) -> Self {
        Error::Core(maps_io_ros::Error::io(context, source))
    }
}
