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
