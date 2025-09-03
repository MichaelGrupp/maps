use std::path::Path;

use eframe::egui;
use fast_image_resize::images::Image as ResizeImage;
use fast_image_resize::{IntoImageView, ResizeOptions, Resizer};
use image::{GenericImageView, ImageBuffer, ImageReader};
use imageproc::map::map_colors_mut;
use log::{debug, info};

#[allow(unused_imports)]
use fast_image_resize::CpuExtensions;

use crate::error::{Error, Result};
use crate::path_helpers::resolve_symlink;

pub fn load_image(path: &Path) -> Result<image::DynamicImage> {
    let path = resolve_symlink(path);
    info!("Loading image: {:?}", path);
    let mut reader = ImageReader::open(&path)
        .map_err(|e| Error::io(format!("Cannot open {:?}", path), e).and_log_it())?;

    reader.no_limits();
    let img = reader
        .decode()
        .map_err(|e| Error::image(format!("Cannot decode {:?}", path), e).and_log_it())?;

    debug!("Loaded image: {:?} {:?}", path, img.dimensions());
    Ok(img)
}

#[cfg(target_arch = "wasm32")]
pub fn load_image_from_bytes(bytes: &[u8]) -> Result<image::DynamicImage> {
    let img_io = ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| Error::io("Cannot create image reader from bytes", e).and_log_it())?;

    let img = img_io
        .decode()
        .map_err(|e| Error::image("Cannot decode image from bytes", e).and_log_it())?;

    debug!("Loaded image from bytes: {:?}", img.dimensions());
    Ok(img)
}

pub fn to_egui_image(img: image::DynamicImage) -> egui::ColorImage {
    let size = [img.width() as usize, img.height() as usize];
    // TODO: rgba might make sense here if we want to use alpha later?
    let pixels = img.to_rgba8().into_raw();
    egui::ColorImage::from_rgba_unmultiplied(size, &pixels)
}

pub fn from_egui_image(egui_img: &egui::ColorImage) -> image::DynamicImage {
    let (width, height) = (egui_img.width() as u32, egui_img.height() as u32);
    let buffer: ImageBuffer<image::Rgba<u8>, _> =
        image::ImageBuffer::from_raw(width, height, egui_img.as_raw().to_vec())
            .expect("failed to convert egui::ColorImage to image::DynamicImage (RGBA8)");
    image::DynamicImage::ImageRgba8(buffer)
}

fn fast_resize(img: &image::DynamicImage, width: u32, height: u32) -> image::DynamicImage {
    let mut resized_img = ResizeImage::new(
        width,
        height,
        img.pixel_type().expect("can't determine pixel type"),
    );
    let mut resizer = Resizer::new();

    let options = ResizeOptions::default();
    resizer
        .resize(img, &mut resized_img, &options)
        .expect("failed to resize image");

    match img.color() {
        image::ColorType::L8 => {
            let buffer: ImageBuffer<image::Luma<u8>, _> =
                image::ImageBuffer::from_raw(width, height, resized_img.into_vec())
                    .expect("failed to create L8 image buffer");
            image::DynamicImage::ImageLuma8(buffer)
        }
        image::ColorType::La8 => {
            let buffer: ImageBuffer<image::LumaA<u8>, _> =
                image::ImageBuffer::from_raw(width, height, resized_img.into_vec())
                    .expect("failed to create La8 image buffer");
            image::DynamicImage::ImageLumaA8(buffer)
        }
        image::ColorType::Rgb8 => {
            let buffer: ImageBuffer<image::Rgb<u8>, _> =
                image::ImageBuffer::from_raw(width, height, resized_img.into_vec())
                    .expect("failed to create Rgb8 image buffer");
            image::DynamicImage::ImageRgb8(buffer)
        }
        image::ColorType::Rgba8 => {
            let buffer: ImageBuffer<image::Rgba<u8>, _> =
                image::ImageBuffer::from_raw(width, height, resized_img.into_vec())
                    .expect("failed to create Rgba8 image buffer");
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

// In-place conversion of all pixels with a color to alpha, if set.
pub fn color_to_alpha(img: &mut image::DynamicImage, color: Option<egui::Color32>) {
    if let Some(color) = color {
        let color = image::Rgba([color.r(), color.g(), color.b(), color.a()]);
        map_colors_mut(img, |c| match c == color {
            true => image::Rgba([0, 0, 0, 0]),
            false => c,
        });
    }
}

pub fn to_rgba8(img: image::DynamicImage) -> image::DynamicImage {
    match img.color() {
        image::ColorType::L8 => image::DynamicImage::from(img.to_rgba8()),
        image::ColorType::La8 => image::DynamicImage::from(img.to_rgba8()),
        image::ColorType::Rgb8 => image::DynamicImage::from(img.to_rgba8()),
        image::ColorType::Rgba8 => img,
        _ => panic!("Unsupported color type: {:?}", img.color()),
    }
}
