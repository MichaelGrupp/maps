use eframe::egui;
use log::{Level, log_enabled};

/// Rotates a rectangle around an origin point and returns the bounding rectangle.
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

/// Calculates the crop intersection of `rect` with `region_of_interest` and
/// quantizes the resulting width and height to an even multiple of `quantize_size`.
/// The resulting rectangle is either equal or one quantum larger than the ROI
/// to ensure that the ROI is always fully covered.
pub fn quantized_intersection(
    rect: &egui::Rect,
    region_of_interest: &egui::Rect,
    quantum: f32,
) -> egui::Rect {
    // First, move the rect to the origin at (0, 0).
    let offset = rect.min.to_vec2();
    let rect = rect.translate(-offset);
    let crop_rect = region_of_interest.translate(-offset);
    let intersection = rect.intersect(crop_rect);

    // Quantize the min point to the nearest lower-or-equal multiple of the quantum
    // if it is affected by the intersection.
    let min = if intersection.min.to_vec2().length() > rect.min.to_vec2().length() {
        let x = (intersection.min.x / quantum).floor() * quantum;
        let y = (intersection.min.y / quantum).floor() * quantum;
        egui::Pos2::new(x, y).max(rect.min)
    } else {
        intersection.min
    };

    // Quantize the max point to the nearest larger-or-equal multiple of the quantum
    // if it is affected by the intersection.
    let max = if intersection.max.to_vec2().length() < rect.max.to_vec2().length() {
        let x = (intersection.max.x / quantum).ceil() * quantum;
        let y = (intersection.max.y / quantum).ceil() * quantum;
        egui::Pos2::new(x, y).min(rect.max)
    } else {
        intersection.max
    };

    // Translate the rect back to its original position.
    egui::Rect::from_min_max(min, max).translate(offset)
}

/// Paints a rectangle with a color when trace logging is enabled.
pub fn debug_paint(ui: &egui::Ui, rect: egui::Rect, color: egui::Color32, label: &str) {
    if !log_enabled!(Level::Trace) {
        return;
    }
    ui.painter().debug_rect(rect, color, label);
}
