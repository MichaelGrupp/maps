use std::vec::Vec;

use eframe::egui;

use crate::meta::Meta;

#[derive(Default)]
pub struct RosMapsApp {
    metas: Vec<Meta>,
    size: [f32; 2],
}

impl RosMapsApp {
    pub fn init(metas: Vec<Meta>, size: [f32; 2]) -> RosMapsApp {
        RosMapsApp { metas, size }
    }
}

impl eframe::App for RosMapsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                let size_per_image = [
                    self.size[0] / self.metas.len() as f32,
                    self.size[1] / self.metas.len() as f32,
                ];
                for meta in &self.metas {
                    let image =
                        egui::Image::new(format!("file://{0}", meta.image_path.to_str().unwrap()));
                    // TODO: Use a better way to scale images.
                    ui.add_sized(size_per_image, image);
                }
            });
        });
    }
}
