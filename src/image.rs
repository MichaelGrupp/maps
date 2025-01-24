use std::path::PathBuf;

use eframe::egui;
use fast_image_resize::images::Image as ResizeImage;
use fast_image_resize::{IntoImageView, ResizeOptions, Resizer};
use image::{GenericImageView, ImageBuffer, ImageReader};
use log::{debug, error, info};

#[allow(unused_imports)]
use fast_image_resize::CpuExtensions;

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
            error!("Error loading image {:?}: {}", path, e.to_string());
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

fn fast_resize(img: &image::DynamicImage, width: u32, height: u32) -> image::DynamicImage {
    let mut resized_img = ResizeImage::new(width, height, img.pixel_type().unwrap());
    let mut resizer = Resizer::new();

    #[allow(unused_unsafe)]
    unsafe {
        // TODO: NEON is shown with artifacts. At least on Apple Silicon M4.
        #[cfg(target_arch = "aarch64")]
        if resizer.cpu_extensions() == CpuExtensions::Neon {
            resizer.set_cpu_extensions(CpuExtensions::None);
        }
    }

    let options = ResizeOptions::default();
    resizer.resize(img, &mut resized_img, &options).unwrap();

    match img.color() {
        image::ColorType::L8 => {
            let buffer: ImageBuffer<image::Luma<u8>, _> =
                image::ImageBuffer::from_raw(width, height, resized_img.into_vec()).unwrap();
            image::DynamicImage::ImageLuma8(buffer)
        }
        image::ColorType::La8 => {
            let buffer: ImageBuffer<image::LumaA<u8>, _> =
                image::ImageBuffer::from_raw(width, height, resized_img.into_vec()).unwrap();
            image::DynamicImage::ImageLumaA8(buffer)
        }
        image::ColorType::Rgb8 => {
            let buffer: ImageBuffer<image::Rgb<u8>, _> =
                image::ImageBuffer::from_raw(width, height, resized_img.into_vec()).unwrap();
            image::DynamicImage::ImageRgb8(buffer)
        }
        image::ColorType::Rgba8 => {
            let buffer: ImageBuffer<image::Rgba<u8>, _> =
                image::ImageBuffer::from_raw(width, height, resized_img.into_vec()).unwrap();
            image::DynamicImage::ImageRgba8(buffer)
        }
        _ => panic!("Unsupported color type: {:?}", img.color()),
    }
}

// Fit the image into the desired size while keeping the aspect ratio.
// Clones the original if the desired size is larger or equal than the original image.
pub fn fit_image(img: &image::DynamicImage, desired_size: egui::Vec2) -> image::DynamicImage {
    let (original_width, original_height) = img.dimensions();
    if (desired_size.x as u32) >= original_width && (desired_size.y as u32) >= original_height {
        return img.clone();
    }
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

    fast_resize(img, new_width, new_height)
}
