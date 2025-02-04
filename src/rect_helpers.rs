use eframe::egui;
use log::{log_enabled, Level};

// Rotates a rectangle around an origin point and returns the bounding rectangle.
pub fn rotate(rect: &egui::Rect, rot: egui::emath::Rot2, origin: egui::Vec2) -> egui::Rect {
    let a = origin + rot * (rect.left_top() - origin.to_pos2());
    let b = origin + rot * (rect.right_top() - origin.to_pos2());
    let c = origin + rot * (rect.left_bottom() - origin.to_pos2());
    let d = origin + rot * (rect.right_bottom() - origin.to_pos2());

    egui::Rect::from_min_max(
        a.min(b).min(c).min(d).to_pos2(),
        a.max(b).max(c).max(d).to_pos2(),
    )
}

// Paints a rectangle with a color when trace logging is enabled.
pub fn debug_paint(ui: &egui::Ui, rect: egui::Rect, color: egui::Color32) {
    if !log_enabled!(Level::Trace) {
        return;
    }
    ui.painter()
        .rect_stroke(rect, 0.0, (1.0, color), egui::StrokeKind::Middle);
}
