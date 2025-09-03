use log::error;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{context}")]
    App { context: String },

    #[error("[IO error] {context} ({source})")]
    Io {
        context: String,
        #[source]
        source: std::io::Error,
    },

    #[error("[Image error] {context} ({source})")]
    Image {
        context: String,
        #[source]
        source: image::ImageError,
    },

    #[error("[YAML error] {context} ({source})")]
    Yaml {
        context: String,
        #[source]
        source: serde_yaml_ng::Error,
    },

    #[error("[TOML error] {context} ({source})")]
    TomlDeserialize {
        context: String,
        #[source]
        source: toml::de::Error,
    },

    #[error("[TOML error] {context} ({source})")]
    TomlSerialize {
        context: String,
        #[source]
        source: toml::ser::Error,
    },
}

/// Macro for implementing error constructors.
macro_rules! impl_error_constructors {
    ($($method_name:ident => $variant:ident, $error_type:ty);* $(;)?) => {
        $(
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
    pub fn app(context: impl ToString) -> Self {
        Self::App {
            context: context.to_string(),
        }
    }

    impl_error_constructors! {
        io => Io, std::io::Error;
        image => Image, image::ImageError;
        yaml => Yaml, serde_yaml_ng::Error;
        toml_deserialize => TomlDeserialize, toml::de::Error;
        toml_serialize => TomlSerialize, toml::ser::Error;
    }
}
