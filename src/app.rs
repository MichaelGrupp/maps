use std::cmp::max;
use std::collections::HashMap;
use std::option::Option;
use std::path::PathBuf;
use std::vec::Vec;

use eframe::egui;
use image::{GenericImageView, ImageReader};
use log::debug;

use crate::meta::Meta;

// Side lengths used for the image pyramid levels.
const SIZES: [u32; 5] = [500, 1000, 2000, 4000, 8000];

struct Pyramid {
    original: egui::ColorImage,
    levels_by_size: HashMap<u32, egui::ColorImage>,
}

impl Pyramid {
    fn new(original: image::DynamicImage) -> Pyramid {
        let original_size = egui::Vec2::new(original.width() as f32, original.height() as f32);
        Pyramid {
            // TODO: avoid cloning the image?
            original: fit_image(original.clone(), original_size),
            levels_by_size: |original: &image::DynamicImage| -> HashMap<u32, egui::ColorImage> {
                let mut levels: HashMap<u32, egui::ColorImage> = HashMap::new();
                for size in SIZES {
                    if max(original.width(), original.height()) < size {
                        // Small enough, no need to create more levels.
                        break;
                    }
                    let level =
                        fit_image(original.clone(), egui::Vec2::new(size as f32, size as f32));
                    levels.insert(size, level);
                }
                levels
            }(&original),
        }
    }

    fn get_level(&self, size: u32) -> &egui::ColorImage {
        // Get the closest size that is larger or equal to the requested size.
        let size = SIZES.iter().find(|&&s| s >= size);
        if size.is_some() {
            return self.levels_by_size.get(&size.unwrap()).unwrap();
        }
        return &self.original;
    }
}

fn load_image(path: &PathBuf) -> image::DynamicImage {
    debug!("Loading image: {:?}", path);
    ImageReader::open(path).unwrap().decode().unwrap()
}

fn load_image_pyramids(metas: &Vec<Meta>) -> Vec<Pyramid> {
    metas
        .iter()
        .map(|meta| Pyramid::new(load_image(&meta.image_path)))
        .collect()
}

fn fit_image(img: image::DynamicImage, desired_size: egui::Vec2) -> egui::ColorImage {
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
    let img = img.resize(new_width, new_height, image::imageops::FilterType::Nearest);
    let size = [img.width() as usize, img.height() as usize];
    // TODO: rgba might make sense here if we want to use alpha later?
    let pixels = img.to_rgba8().into_raw();
    egui::ColorImage::from_rgba_unmultiplied(size, &pixels)
}

#[derive(Default)]
pub struct RosMapsApp {
    metas: Vec<Meta>,
    image_pyramids: Vec<Pyramid>,
    texture_handles: Vec<Option<egui::TextureHandle>>,
    desired_size: egui::Vec2,
}

impl RosMapsApp {
    pub fn init(metas: Vec<Meta>) -> RosMapsApp {
        RosMapsApp {
            // TODO: probably makes more sense to work with maps here.
            texture_handles: vec![None; metas.len()],
            image_pyramids: load_image_pyramids(&metas),
            metas: metas,
            desired_size: egui::Vec2::default(), // Set in show_images.
        }
    }

    fn update_desired_size(&mut self, ui: &egui::Ui) {
        let pixels_per_point = ui.ctx().zoom_factor() * ui.ctx().pixels_per_point();
        // TODO: this is probably not the exact size we want.
        let viewport_info = ui.ctx().screen_rect();
        let desired_size = egui::vec2(
            viewport_info.width() * pixels_per_point,
            viewport_info.height() * pixels_per_point,
        );
        let threshold = 5.;
        if (desired_size.x - self.desired_size.x).abs() > threshold
            || (desired_size.y - self.desired_size.y).abs() > threshold
        {
            // Clear the texture handles if the size changes "significantly".
            // Note that in egui dropping the last handle of a texture will free it.
            debug!(
                "Desired size changed to {:?}, clearing textures.",
                desired_size
            );
            self.texture_handles = vec![None; self.metas.len()];
        }
        self.desired_size = desired_size;
    }

    fn show_images(&mut self, ui: &mut egui::Ui) {
        self.update_desired_size(ui);
        for (i, texture_handle) in self.texture_handles.iter_mut().enumerate() {
            let texture: &egui::TextureHandle = texture_handle.get_or_insert_with(|| {
                // Load the texture only if needed.
                let name: &str = self.metas[i].image_path.to_str().unwrap();
                debug!("Loading texture for: {}", name);
                debug!(
                    "Image pyramid levels: {}",
                    self.image_pyramids[i].levels_by_size.len()
                );
                let image_pyramid = &self.image_pyramids[i];
                ui.ctx().load_texture(
                    name,
                    image_pyramid
                        .get_level(self.desired_size.max_elem() as u32)
                        .clone(),
                    Default::default(),
                )
            });
            ui.image(texture);
        }
    }
}

impl eframe::App for RosMapsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                self.show_images(ui);
            });
        });
    }
}
