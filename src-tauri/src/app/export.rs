use std::fs;
use std::path::{Path, PathBuf};
use rfd::FileDialog;
use tauri::{command, WebviewUrl, Window};
use crate::app::{console, mod_installation, mods, profiles};
use crate::app::profiles::Profile;
use crate::app::utility::{paths, zips};

#[command]
pub async fn open_export(handle: tauri::AppHandle) {
    tauri::WebviewWindowBuilder::new(
        &handle,
        "Exporter",
        WebviewUrl::App("/exporter".into())
    ).title("Export")
        .min_inner_size(600.0, 350.0)
        .inner_size(600.0, 350.0)
        .transparent(true)
        .build()
        .unwrap();
}

#[command]
pub async fn close_export(window: Window) {
    window.close().unwrap();
}

#[command]
pub fn select_export_dir() -> String {
    let file = FileDialog::new()
        .set_directory(".")
        .pick_folder();

    if let Some(path) = file {
        path.to_string_lossy().to_string()
    } else {
        "".to_string()
    }
}

#[command]
pub fn export_profile(handle: tauri::AppHandle, name: &str, path: &str) -> bool {
    if name == "All Profiles" {
        export_all(handle, path);
        true
    } else {
        export_one(handle, name, path);
        true
    }
}

fn export_one(handle: tauri::AppHandle, name: &str, path: &str) {
    let profiles = profiles::get_profiles();
    let mut profile: Profile = Profile {
        name: "".to_string(),
        mods: vec![],
        currently: false,
    };

    for prof in profiles {
        if prof.name == name {
            profile = prof;
        }
    }

    let export_path = format!("export_{}", profile.name);

    let mut export_dir = paths::temp_path();
    export_dir.push(&export_path);
    fs::create_dir_all(&export_dir).unwrap();

    for mod_info in &profile.mods {
        let mut mod_path = paths::mod_path();
        mod_path.push(format!("{}.zip", mod_info.name));
        let mut export_mod_path = export_dir.clone();
        export_mod_path.push(format!("{}.zip", mod_info.name));
        fs::copy(&mod_path, &export_mod_path).unwrap();
    }

    let export_zip_path = Path::new(path).join(format!("export_{}.zip", &profile.name));

    let profiles: Vec<Profile> = vec![profile];
    let profile_json = serde_json::to_string(&profiles).unwrap();
    let profile_json_path = export_dir.clone();
    fs::write(profile_json_path.join("profile.json"), profile_json).unwrap();

    zips::zip_directory(&export_dir, &export_zip_path).unwrap();
    console::add_line(&handle, "<span class=\"console-green\">[Junimo] Exported profile</span>".to_string());
}

fn export_all(handle: tauri::AppHandle, path: &str) {
    let mut export_dir = paths::temp_path();
    export_dir.push("export_all");
    fs::create_dir_all(&export_dir).unwrap();

    for mod_info in &mods::get_all_mods() {
        let mut mod_path = paths::mod_path();
        mod_path.push(format!("{}.zip", mod_info.name));
        let mut export_mod_path = export_dir.clone();
        export_mod_path.push(format!("{}.zip", mod_info.name));
        fs::copy(&mod_path, &export_mod_path).unwrap();
    }

    let export_zip_path = Path::new(path).join("export_all.zip");

    let profile_file = paths::appdata_path().join("profile.json");
    let mods_file = paths::appdata_path().join("mods.json");
    fs::copy(&profile_file, export_dir.join("profile.json")).unwrap();
    fs::copy(&mods_file, export_dir.join("mods.json")).unwrap();

    zips::zip_directory(&export_dir, &export_zip_path).unwrap();
    console::add_line(&handle, "<span class=\"console-green\">[Junimo] Exported all profiles and mods</span>".to_string());
}