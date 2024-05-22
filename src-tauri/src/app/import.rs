use crate::app::profiles::Profile;
use crate::app::utility::{paths, zips};
use crate::app::{console, mods, profiles};
use rfd::FileDialog;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::{fs, io, thread};
use tauri::{command, Manager, WebviewUrl, Window};
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

/// Opens the import window
///
/// * `handle` - The app handle of the Tauri application
#[command]
pub async fn open_import(handle: tauri::AppHandle) {
    #[cfg(target_os = "windows")]
    tauri::WebviewWindowBuilder::new(&handle, "Importer", WebviewUrl::App("/importer".into()))
        .title("Import")
        .min_inner_size(600.0, 350.0)
        .inner_size(600.0, 350.0)
        .transparent(true)
        .build()
        .unwrap();

    #[cfg(target_os = "unix")]
    tauri::WebviewWindowBuilder::new(&handle, "Importer", WebviewUrl::App("/importer".into()))
        .title("Import")
        .min_inner_size(600.0, 350.0)
        .inner_size(600.0, 350.0)
        .build()
        .unwrap();
}

/// Opens a file dialog to select a directory to import
///
/// # Returns the path of the selected directory, else an empty string
#[command]
pub fn select_import_dir() -> String {
    let file = FileDialog::new()
        .add_filter("zip", &["zip"])
        .set_directory("/")
        .pick_file();

    if let Some(path) = file {
        path.to_string_lossy().to_string()
    } else {
        "".to_string()
    }
}

/// Imports a profile from a zip file
///
/// * `window` - The window to close after importing
/// * `handle` - The app handle of the Tauri application
/// * `path` - The path of the zip file to import
/// * `all` - Whether the import is an "all profiles" import
#[command]
pub fn import_profile(window: Window, handle: tauri::AppHandle, path: &str, all: bool) {
    let path = path.clone().to_string();
    let all = all.clone();
    let window_clone = window.clone();

    // Spawn a new thread to import the profile in the background
    thread::spawn(move || {
        let path_as_buff = Path::new(&path);
        let file = fs::File::open(&path_as_buff).unwrap();
        let mut file_name = path_as_buff
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        file_name = file_name.replace(".zip", "");

        let zip_archive = zip::ZipArchive::new(file).unwrap();
        let import_result = zips::import_zip(zip_archive, &paths::appdata_path(), &paths::temp_path(), all);
        match import_result {
            Ok(_) => {
                console::add_line(
                    &handle,
                    format!(
                        "<span class=\"console-green\">[Junimo] Imported profile(s) {}</span>",
                        file_name
                    ),
                );
            }
            Err(e) => {
                console::add_line(
                    &handle,
                    format!(
                        "<span class=\"console-green\">[Junimo] Failed to import profile {}: {}</span>",
                        file_name,
                        e
                    ),
                );
            }
        }
        window_clone.close().unwrap();
    });
}