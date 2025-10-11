//! Error handling for the `maps_io_ros` crate.

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

/// Error types for the `maps_io_ros` crate.
/// Allows to wrap external errors in a unified way.
#[derive(Error, Debug)]
pub enum Error {
    /// An I/O error with additional context.
    #[error("[IO error] {context} ({source})")]
    Io {
        context: String,
        #[source]
        source: std::io::Error,
    },

    /// An image loading error with additional context.
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
}

/// Macro for generating wrapping error constructors with doc comments.
#[macro_export]
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
    // Generate the wrapping error constructors.
    impl_error_constructors! {
        io => Io, std::io::Error;
        image => Image, image::ImageError;
        yaml => Yaml, serde_yaml_ng::Error;
    }
}
