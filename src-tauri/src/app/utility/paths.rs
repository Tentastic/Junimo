use std::fs;
use std::path::PathBuf;
use crate::app::utility::paths;

pub fn appdata_path() -> PathBuf {
    let mut config_path = tauri::api::path::config_dir().unwrap();
    config_path.push("Junimo");
    config_path
}

pub fn mod_path() -> PathBuf {
    let mut mods_path = tauri::api::path::config_dir().unwrap();
    mods_path.push("Junimo");
    mods_path.push("mods");
    mods_path
}

pub fn mod_json_path() -> PathBuf {
    let mut mods_path = tauri::api::path::config_dir().unwrap();
    mods_path.push("Junimo");
    mods_path.push("mods.json");
    mods_path
}

pub fn temp_path() -> PathBuf {
    let mut temp_path = std::env::temp_dir();
    temp_path.push("Junimo");
    fs::create_dir_all(&temp_path).unwrap();
    temp_path
}