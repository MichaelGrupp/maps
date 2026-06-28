use eframe::egui;

/// A clickable header for controlling a `collapsed` state.
/// Intended as a cheap way to control areas within grid layout sections,
/// where egui::CollapsingHeader doesn't work easily or messes up the layout.
/// Returns true if the section is open (not collapsed).
pub(crate) fn section_heading(ui: &mut egui::Ui, heading: &str, collapsed: &mut bool) -> bool {
    let collapsed_icon = if *collapsed { "⏵ " } else { "" };
    let tooltip_action = if *collapsed { "expand" } else { "collapse" };
    if ui
        .heading(format!("{collapsed_icon}{heading}"))
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(format!("Click to {tooltip_action} this section."))
        .clicked()
    {
        *collapsed = !*collapsed;
    }
    !*collapsed
}

/// A left-aligned label occupying exactly `width`.
pub(crate) fn fixed_label(ui: &mut egui::Ui, width: f32, text: &str) -> egui::Response {
    let size = egui::vec2(width, ui.spacing().interact_size.y);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let mut cell = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    cell.label(text)
}

/// Lays out `(label, hover)` buttons in equal-width, full-width columns so
/// related buttons line up neatly. Returns the clicked button's index, if any.
pub(crate) fn button_row(ui: &mut egui::Ui, buttons: &[(&str, &str)]) -> Option<usize> {
    let mut clicked = None;
    ui.columns(buttons.len(), |columns| {
        for (i, (label, hover)) in buttons.iter().enumerate() {
            columns[i].vertical_centered_justified(|ui| {
                if ui.button(*label).on_hover_text(*hover).clicked() {
                    clicked = Some(i);
                }
            });
        }
    });
    clicked
}

/// Shortens a path for UI usage if it's desired.
pub(crate) fn display_path(path: &str, show_full_paths: bool) -> String {
    if show_full_paths {
        return path.to_string();
    }
    std::path::Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(path)
        .to_string()
}

pub(crate) fn monospace(text: impl Into<String>) -> egui::RichText {
    egui::RichText::new(text).monospace()
}
