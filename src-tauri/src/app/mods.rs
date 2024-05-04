use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use rfd::FileDialog;
use zip::read::ZipArchive;
use std::{env, fs};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, command, Manager};
use crate::app::{api, profiles};



#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModInfo {
    pub name: String,
    summary: String,
    description: String,
    picture_url: String,
    mod_downloads: u64,
    mod_unique_downloads: u64,
    uid: u64,
    mod_id: u32,
    game_id: u32,
    allow_rating: bool,
    domain_name: String,
    category_id: u32,
    pub version: String,
    endorsement_count: u32,
    created_timestamp: u64,
    created_time: String,
    updated_timestamp: u64,
    updated_time: String,
    author: String,
    uploaded_by: String,
    uploaded_users_profile_url: String,
    contains_adult_content: bool,
    status: String,
    available: bool,
}

impl PartialEq for ModInfo {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

pub fn get_path() -> PathBuf {
    let mut mods_path = tauri::api::path::config_dir().unwrap();
    mods_path.push("Junimo");
    mods_path.push("mods");
    mods_path
}

pub fn mods_path() -> PathBuf {
    let mut mods_path = tauri::api::path::config_dir().unwrap();
    mods_path.push("Junimo");
    mods_path.push("mods.json");
    mods_path
}

pub fn save_mods(mod_infos: Vec<ModInfo>) {
    let json = serde_json::to_string(&mod_infos).unwrap();
    let mut file = File::create(mods_path()).expect("Fail");
    file.write_all(json.as_bytes()).unwrap();
}

#[command]
pub fn get_mods() -> Vec<ModInfo> {
    let path = mods_path();
    let current_profile = profiles::get_current_profile();

    if !path.exists() {
        let mods: Vec<ModInfo> = Vec::new();
        save_mods(mods);
    }

    let all_mods = get_all_mods();
    all_mods.iter().filter(|mod_info| !current_profile.mods.contains(mod_info)).cloned().collect()
}

#[command]
pub fn get_all_mods() -> Vec<ModInfo> {
    let path = mods_path();

    if !path.exists() {
        let mods: Vec<ModInfo> = Vec::new();
        save_mods(mods);
    }

    let data_raw = tauri::api::file::read_string(path).unwrap();
    let data = data_raw.as_str();
    serde_json::from_str(data).unwrap()
}

#[command]
pub fn add_mod() {
    let file = FileDialog::new()
        .add_filter("zip", &["zip"])
        .add_filter("rar", &["rar"])
        .set_directory("/")
        .pick_file();

    if let Some(path) = file {
        let zip_file_path = path;
        let output_folder_path = get_path();

        let zip_file = File::open(&zip_file_path).unwrap();
        let zip_archive = ZipArchive::new(zip_file).unwrap();

        extract_zip(zip_archive, &output_folder_path).unwrap();
    } else {
        println!("No file was selected.");
    }
}

#[command]
pub fn uninstall_mod(mod_name: &str) {
    let mut mods = get_mods();
    mods.retain(|mod_info| mod_info.name != mod_name);
    save_mods(mods);
}

fn extract_zip<R: io::Read + io::Seek>(mut archive: ZipArchive<R>, destination: &Path) -> zip::result::ZipResult<()> {
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => destination.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            if !outpath.exists() {
                fs::create_dir_all(&outpath)?;
            }
        } else {
            if let Some(parent) = outpath.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }

        // Set the permissions if on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode))?;
            }
        }
    }
    Ok(())
}