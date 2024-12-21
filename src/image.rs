use std::path::PathBuf;

use eframe::egui;
use image::{GenericImageView, ImageReader};
use log::{debug, error, info};

pub fn load_image(path: &PathBuf) -> Result<image::DynamicImage, image::ImageError> {
    info!("Loading image: {:?}", path);
    match ImageReader::open(path) {
        Ok(reader) => match reader.decode() {
            Ok(img) => {
                debug!("Loaded image: {:?} {:?}", path, img.dimensions());
                Ok(img)
            }
            Err(e) => {
                error!("Error decoding image: {:?}", e);
                Err(e)
            }
        },
        Err(e) => {
            error!("Error loading image: {:?}", e);
            Err(image::ImageError::IoError(e))
        }
    }
}

pub fn to_egui_image(img: image::DynamicImage) -> egui::ColorImage {
    let size = [img.width() as usize, img.height() as usize];
    // TODO: rgba might make sense here if we want to use alpha later?
    let pixels = img.to_rgba8().into_raw();
    egui::ColorImage::from_rgba_unmultiplied(size, &pixels)
}

// Fit the image into the desired size while keeping the aspect ratio.
pub fn fit_image(img: image::DynamicImage, desired_size: egui::Vec2) -> image::DynamicImage {
    let (original_width, original_height) = img.dimensions();
    let aspect_ratio = original_width as f32 / original_height as f32;
    let (new_width, new_height) = if desired_size.x / desired_size.y > aspect_ratio {
        (
            (desired_size.y * aspect_ratio) as u32,
            desired_size.y as u32,
        )
    } else {
        (
            desired_size.x as u32,
            (desired_size.x / aspect_ratio) as u32,
        )
    };
    img.resize(new_width, new_height, image::imageops::FilterType::Nearest)
}
