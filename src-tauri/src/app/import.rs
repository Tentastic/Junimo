use std::{fs, io};
use std::fs::File;
use std::path::{Path, PathBuf};
use rfd::FileDialog;
use tauri::{command, Manager, WebviewUrl, Window};
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;
use crate::app::{console, mods, profiles};
use crate::app::profiles::Profile;
use crate::app::utility::{paths, zips};

#[command]
pub async fn open_import(handle: tauri::AppHandle) {
    tauri::WebviewWindowBuilder::new(
        &handle,
        "Importer",
        WebviewUrl::App("/importer".into())
    ).title("Import")
        .min_inner_size(600.0, 350.0)
        .inner_size(600.0, 350.0)
        .transparent(true)
        .build()
        .unwrap();
}

#[command]
pub async fn close_import(window: Window) {
    window.close().unwrap();
}

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

#[command]
pub fn import_profile(handle: tauri::AppHandle, path: &str, all: bool) -> bool {
    let path_as_buff = Path::new(path);
    let file = fs::File::open(&path_as_buff).unwrap();
    let mut file_name = path_as_buff.file_name().unwrap().to_str().unwrap().to_string();
    file_name = file_name.replace(".zip", "");

    let mut temp_path = paths::temp_path();
    temp_path.push(file_name);
    fs::create_dir_all(&temp_path).unwrap();

    let mut zip_archive = zip::ZipArchive::new(file).unwrap();
    zips::extract_zip(zip_archive, &temp_path).unwrap();

    if all {
        import_all(&handle, &temp_path);
    } else {
        import_one(&handle, &temp_path);
    }
    &handle.emit("reload", true).unwrap();
    true
}

fn import_all(handle: &tauri::AppHandle, temp_path: &PathBuf) {
    let walkdir = WalkDir::new(&temp_path);
    let it = walkdir.into_iter();

    for entry in it.filter_map(|e| e.ok()) {
        let mut junimo_path = paths::appdata_path();
        let mut junimo_mod_path = paths::mod_path();

        let path = entry.path();

        if path.file_name().unwrap().to_string_lossy().contains(".zip") {
            junimo_mod_path.push(path.file_name().unwrap().to_string_lossy().to_string());
            fs::copy(&path, &junimo_mod_path).unwrap();
        }
        else if path.file_name().unwrap().to_string_lossy().contains(".json") {
            junimo_path.push(path.file_name().unwrap().to_string_lossy().to_string());
            fs::copy(&path, &junimo_path).unwrap();
        }
    }
    fs::remove_dir_all(&temp_path).unwrap();
    let profiles = profiles::get_profiles();
    &handle.emit("profile-update", &profiles).unwrap();
    console::add_line(&handle, "<span class=\"console-green\">[Junimo] Imported all profiles</span>".to_string());
}

fn import_one(handle: &tauri::AppHandle, temp_path: &PathBuf) {
    let mut mods = mods::get_all_mods();
    let mut profiles = profiles::get_profiles();

    let walkdir = WalkDir::new(&temp_path);
    let it = walkdir.into_iter();

    for entry in it.filter_map(|e| e.ok()) {
        let mut junimo_path = paths::appdata_path();
        let mut junimo_mod_path = paths::mod_path();

        let path = entry.path();

        if path.file_name().unwrap().to_string_lossy().contains(".zip") {
            junimo_mod_path.push(path.file_name().unwrap().to_string_lossy().to_string());
            let mut check_path = junimo_mod_path.clone();
            check_path.push(path.file_name().unwrap().to_string_lossy().to_string());
            if !check_path.exists() {
                fs::copy(&path, &junimo_mod_path).unwrap();
            }
        }
        else if path.file_name().unwrap().to_string_lossy().contains("profile.json") {
            let data_raw = fs::read_to_string(path).unwrap();
            let data = data_raw.as_str();
            let import_profile: Vec<Profile> = serde_json::from_str(data).unwrap();
            let mut cloned_profile = import_profile[0].clone();
            cloned_profile.currently = false;
            profiles.insert(1, cloned_profile);

            for mod_info in &import_profile[0].mods {
                if (mods.iter().find(|&x| x.name == mod_info.name)).is_none() {
                    mods.push(mod_info.clone());
                }
            }
        }
    }
    profiles::save_profiles(&profiles);
    mods::save_mods(mods);
    fs::remove_dir_all(&temp_path).unwrap();
    &handle.emit("profile-update", &profiles).unwrap();
    console::add_line(&handle, "<span class=\"console-green\">[Junimo] Imported profile</span>".to_string());
}