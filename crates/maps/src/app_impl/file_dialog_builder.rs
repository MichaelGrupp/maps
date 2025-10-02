//! Simple builders for creating file dialogs with some defaults used by this crate.

use std::{env::current_dir, path::PathBuf, sync::Arc};

use eframe::egui;
use egui_file_dialog::FileDialog;

/// Creates a file dialog for YAML files.
pub fn yaml(initial_dir: Option<&PathBuf>) -> FileDialog {
    FileDialogBuilder::new(initial_dir)
        .with_filter("YAML", vec!["yaml", "yml"])
        .build()
}

/// Creates a file dialog for TOML files.
pub fn toml(initial_dir: Option<&PathBuf>) -> FileDialog {
    FileDialogBuilder::new(initial_dir)
        .with_filter("TOML", vec!["toml"])
        .build()
}

/// Creates a file dialog for PNG files.
pub fn png(initial_dir: Option<&PathBuf>) -> FileDialog {
    FileDialogBuilder::new(initial_dir)
        .with_filter("PNG", vec!["png"])
        .build()
}

pub(crate) struct FileDialogBuilder {
    dialog: FileDialog,
}

impl FileDialogBuilder {
    pub fn new(initial_dir: Option<&PathBuf>) -> Self {
        FileDialogBuilder {
            dialog: FileDialog::new()
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0., 0.))
                .initial_directory(
                    initial_dir
                        .cloned()
                        .unwrap_or(current_dir().expect("wtf no cwd??")),
                ),
        }
    }

    pub fn with_filter<'a>(mut self, name: &str, suffixes: impl Into<Vec<&'a str>>) -> Self {
        let suffixes_vec: Vec<String> = suffixes.into().iter().map(|s| s.to_string()).collect();
        self.dialog = self
            .dialog
            .add_file_filter(
                name,
                Arc::new(move |path| {
                    suffixes_vec.iter().any(|suffix| {
                        path.extension()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or_default()
                            .ends_with(suffix)
                    })
                }),
            )
            .default_file_filter(name);

        self
    }

    pub fn build(self) -> FileDialog {
        self.dialog
    }
}
