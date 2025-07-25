use eframe::egui::ecolor::{gamma_u8_from_linear_f32, linear_f32_from_gamma_u8};
use eframe::egui::Color32;

/// Migrates a color that was serialized with egui < 0.32.
/// This is a best-effort attempt to reconstruct the original unmultiplied color
/// (which gets now interpreted as premultiplied) and create a new Color32 from it
/// using the proper `from_rgba_unmultiplied` constructor.
/// Due to general changes in how colors & blending are handled in egui, the result
/// can still appear visually different from the original color.
/// Related: https://github.com/emilk/egui/pull/5824
/// https://github.com/emilk/egui/pull/7311
pub(crate) fn migrate_old_egui_color(old_color: Option<Color32>) -> Option<Color32> {
    let Some(old_color) = old_color else {
        return None;
    };
    let [r, g, b, a] = old_color.to_array();

    // Handle special cases where migration isn't needed.
    // Fully transparent or opaque colors are fine.
    if a == 0 || a == 255 {
        return Some(old_color);
    }

    // Convert premultiplied gamma values to linear space.
    let a_linear = a as f32 / 255.0;
    let r_linear_premult = linear_f32_from_gamma_u8(r);
    let g_linear_premult = linear_f32_from_gamma_u8(g);
    let b_linear_premult = linear_f32_from_gamma_u8(b);

    // Unmultiply in linear space.
    let r_linear_unmult = r_linear_premult / a_linear;
    let g_linear_unmult = g_linear_premult / a_linear;
    let b_linear_unmult = b_linear_premult / a_linear;

    // Convert back to gamma space.
    let r_gamma_unmult = gamma_u8_from_linear_f32(r_linear_unmult);
    let g_gamma_unmult = gamma_u8_from_linear_f32(g_linear_unmult);
    let b_gamma_unmult = gamma_u8_from_linear_f32(b_linear_unmult);

    Some(Color32::from_rgba_unmultiplied(
        r_gamma_unmult,
        g_gamma_unmult,
        b_gamma_unmult,
        a,
    ))
}
