//! Functions for loading / saving images.

use std::path::Path;

use image::ImageReader;

use crate::error::{Error, Result};
use crate::os_helpers::resolve_symlink;

/// Load an image from the given path.
/// Symlinks are resolved automatically.
pub fn load_image(path: &Path) -> Result<image::DynamicImage> {
    let path = resolve_symlink(path);
    let mut reader =
        ImageReader::open(&path).map_err(|e| Error::io(format!("Cannot open {path:?}"), e))?;

    reader.no_limits();
    let img = reader
        .decode()
        .map_err(|e| Error::image(format!("Cannot decode {path:?}"), e))?;

    Ok(img)
}

/// Load an image from a bytes stream (e.g. in wasm applications).
/// The image format is guessed automatically.
pub fn load_image_from_bytes(bytes: &[u8]) -> Result<image::DynamicImage> {
    let img_io = ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| Error::io("Cannot create image reader from bytes", e))?;

    let img = img_io
        .decode()
        .map_err(|e| Error::image("Cannot decode image from bytes", e))?;

    Ok(img)
}

/// Save an image to the given path.
/// The image format is inferred from the file extension.
pub fn save_image(path: &Path, img: &image::DynamicImage) -> Result<()> {
    img.save(path)
        .map_err(|e| crate::Error::image(format!("Failed to save image to {path:?}"), e))?;
    Ok(())
}
