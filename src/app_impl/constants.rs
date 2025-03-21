pub const SPACE: f32 = 10.;
pub const ICON_SIZE: f32 = 20.;

/// Whether to fit our header panel into the window title bar.
/// This frees a bit of space and looks more elegant. Available only on macOS.
pub const CUSTOM_TITLEBAR: bool = cfg!(target_os = "macos");
/// Indentation to make room for window controls when a custom titlebar is used.
pub const HEADER_PANEL_INDENT: f32 = if CUSTOM_TITLEBAR { 65. } else { 0. };
