#[cfg(target_os = "linux")]
use {
    crate::error::{Error, Result},
    log::{debug, info},
};

#[cfg(target_os = "linux")]
/// Writes a .desktop file with icon for Linux desktop environments,
/// pointing to the current executable.
/// This allows to launch the app from the application menu or desktop.
///
/// Doesn't overwrite an existing file unless `overwrite` is true.
/// `app_id` has to match the app ID of the eframe app on Wayland, see also:
/// https://github.com/emilk/egui/issues/3992#issuecomment-3067278124
pub fn write_desktop_file(app_id: &str, overwrite: bool) -> Result<()> {
    let Some(home_dir) = std::env::home_dir() else {
        return Err(Error::app(
            "Cannot determine home directory to write desktop file.",
        ));
    };
    let exec_path = std::env::current_exe()
        .map_err(|e| Error::io("Cannot determine executable path to write desktop file", e))?;
    if !exec_path.exists() {
        return Err(Error::app(format!(
            "Executable path {exec_path:?} does not exist. Cannot write desktop file."
        )));
    }
    let local_share_app_dir = home_dir.join(".local/share/applications");
    if !local_share_app_dir.exists()
        && let Err(e) = std::fs::create_dir_all(&local_share_app_dir)
    {
        return Err(Error::app(format!(
            "Cannot create directory {local_share_app_dir:?} to write desktop file: {e}"
        )));
    }
    let icon_path = home_dir.join(".local/share/applications/maps_icon.png");
    let desktop_entry = format!(
        "[Desktop Entry]\n\
         Name={app_id}\n\
         Comment=Inspect, compare and align multiple grid maps in an intuitive & fast GUI\n\
         Exec={}\n\
         Icon={}\n\
         Type=Application\n\
         Categories=maps;ROS;SLAM;Navigation;Development;Engineering;Rust;egui\n\
         StartupNotify=false\n",
        exec_path.to_str().expect("non UTF-8 exec_path!"),
        icon_path.to_str().expect("non UTF-8 icon_path!")
    );
    let desktop_file_path = local_share_app_dir.join(format!("{app_id}.desktop"));
    if !overwrite && desktop_file_path.exists() {
        debug!("Not overwriting existing desktop file at {desktop_file_path:?}");
        return Ok(());
    }
    if let Err(e) = std::fs::write(&desktop_file_path, desktop_entry) {
        return Err(Error::app(format!(
            "Cannot write desktop file to {desktop_file_path:?}: {e}"
        )));
    }
    let icon_data = include_bytes!("../data/icon.png");
    if let Err(e) = std::fs::write(&icon_path, icon_data) {
        return Err(Error::app(format!(
            "Cannot write icon file to {icon_path:?}: {e}"
        )));
    }
    info!("Wrote desktop file to {desktop_file_path:?}");
    Ok(())
}
