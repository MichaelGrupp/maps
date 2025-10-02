use thiserror::Error;

use maps_io_ros::impl_error_constructors;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {context}")]
    Io {
        context: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Image error: {context}")]
    Image {
        context: String,
        #[source]
        source: image::ImageError,
    },
}

impl Error {
    // Generate the wrapping error constructors.
    impl_error_constructors! {
        io => Io, std::io::Error;
        image => Image, image::ImageError;
    }
}
