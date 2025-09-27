pub const SPACE: f32 = 10.;
pub const ICON_SIZE: f32 = 20.;

/// Whether it's possible to fit our header panel into the window title bar.
/// This frees a bit of space and looks more elegant. Available only on macOS.
pub const CUSTOM_TITLEBAR_SUPPORTED: bool = cfg!(target_os = "macos");
/// Indentation to make room for window controls when a custom titlebar is used.
pub const HEADER_PANEL_INDENT: f32 = 75.; // 3 Mac buttons + padding
