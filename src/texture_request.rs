use eframe::egui;

#[derive(Debug)]
pub struct TextureRequest {
    pub client: String,
    pub desired_rect: egui::Rect,
}

impl TextureRequest {
    pub fn new(client: String, desired_rect: egui::Rect) -> TextureRequest {
        TextureRequest {
            client,
            desired_rect,
        }
    }
}

#[derive(Debug)]
pub struct CropRequest {
    pub uncropped: TextureRequest,
    pub visible_rect: egui::Rect,
    pub uv: [egui::Pos2; 2],
}

impl CropRequest {
    pub fn from_visible(ui: &egui::Ui, uncropped: TextureRequest) -> CropRequest {
        let viewport_rect = ui.clip_rect();
        let image_rect = uncropped.desired_rect;
        let visible_rect = image_rect.intersect(viewport_rect);
        CropRequest {
            uncropped,
            visible_rect,
            uv: [
                egui::Pos2::new(
                    (visible_rect.min.x - image_rect.min.x) / image_rect.width(),
                    (visible_rect.min.y - image_rect.min.y) / image_rect.height(),
                ),
                egui::Pos2::new(
                    (visible_rect.max.x - image_rect.min.x) / image_rect.width(),
                    (visible_rect.max.y - image_rect.min.y) / image_rect.height(),
                ),
            ],
        }
    }
}
