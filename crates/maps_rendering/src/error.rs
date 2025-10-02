use std::fmt;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {message}")]
    Io {
        message: String,
        #[source]
        source: std::io::Error,
    },
    #[error("Image error: {message}")]
    Image {
        message: String,
        #[source]
        source: image::ImageError,
    },
}

impl Error {
    pub fn io(message: impl fmt::Display, source: std::io::Error) -> Self {
        Self::Io {
            message: message.to_string(),
            source,
        }
    }

    pub fn image(message: impl fmt::Display, source: image::ImageError) -> Self {
        Self::Image {
            message: message.to_string(),
            source,
        }
    }
}
