use thiserror::Error;

use maps_io_ros::impl_error_constructors;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    /// Error from maps_io_ros (I/O, YAML parsing, etc.)
    #[error(transparent)]
    Core(#[from] maps_io_ros::Error),

    #[error("[Image error] {context} ({source})")]
    Image {
        context: String,
        #[source]
        source: image::ImageError,
    },
}

impl Error {
    /// Create I/O errors by delegating to maps_io_ros
    pub fn io(context: impl ToString, source: std::io::Error) -> Self {
        Error::Core(maps_io_ros::Error::io(context, source))
    }

    // Generate the wrapping error constructors.
    impl_error_constructors! {
        image => Image, image::ImageError;
    }
}
