//! Error handling for the `maps` crate.

use log::error;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

/// Error types for the `maps` crate.
/// Allows to handle app errors and wrapped external errors in a unified way.
#[derive(Error, Debug)]
pub enum Error {
    /// A `maps`-internal and application-specific error.
    #[error("{message}")]
    App { message: String },

    /// An I/O error with additional context.
    #[error("[IO error] {context} ({source})")]
    Io {
        context: String,
        #[source]
        source: std::io::Error,
    },

    /// Image loading or processing error with additional context.
    #[error("[Image error] {context} ({source})")]
    Image {
        context: String,
        #[source]
        source: image::ImageError,
    },

    /// YAML serialization or deserialization error with additional context.
    #[error("[YAML error] {context} ({source})")]
    Yaml {
        context: String,
        #[source]
        source: serde_yaml_ng::Error,
    },

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

/// Macro for generating wrapping error constructors with doc comments.
macro_rules! impl_error_constructors {
    ($($method_name:ident => $variant:ident, $error_type:ty);* $(;)?) => {
        $(
            #[doc = concat!("Wrap a `", stringify!($error_type), "` with additional context message.")]
            pub fn $method_name(context: impl ToString, source: $error_type) -> Self {
                Self::$variant {
                    context: context.to_string(),
                    source,
                }
            }
        )*
    };
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
        io => Io, std::io::Error;
        image => Image, image::ImageError;
        yaml => Yaml, serde_yaml_ng::Error;
        toml_deserialize => TomlDeserialize, toml::de::Error;
        toml_serialize => TomlSerialize, toml::ser::Error;
    }
}
