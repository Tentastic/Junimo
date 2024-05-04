use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use tauri::command;
use crate::app::mods::{ModInfo};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Profile {
    name: String,
    pub mods: Vec<ModInfo>,
    currently: bool
}

pub fn profile_path() -> PathBuf {
    let mut mods_path = tauri::api::path::config_dir().unwrap();
    mods_path.push("Junimo");
    mods_path.push("profile.json");
    mods_path
}

pub fn save_profiles(profiles: Vec<Profile>) {
    let json = serde_json::to_string(&profiles).unwrap();
    let mut file = File::create(profile_path()).expect("Fail");
    file.write_all(json.as_bytes()).unwrap();
}

pub fn get_current_profile() -> Profile {
    let path = profile_path();
    let data_raw = tauri::api::file::read_string(path).unwrap();
    let data = data_raw.as_str();
    let profiles: Vec<Profile> = serde_json::from_str(data).unwrap();

    for profile in &profiles {
        if profile.currently {
            return profile.clone();
        }
    }

    profiles[0].clone()
}

#[command]
pub fn get_profile() -> Vec<Profile> {
    let path = profile_path();

    if !path.exists() {
        let mut profiles: Vec<Profile> = Vec::new();
        let profile = Profile {
            name: "Default".to_string(),
            mods: Vec::new(),
            currently: true
        };
        profiles.push(profile);
        save_profiles(profiles);
    }

    let data_raw = tauri::api::file::read_string(path).unwrap();
    let data = data_raw.as_str();
    serde_json::from_str(data).unwrap()
}

#[command]
pub fn change_profile_mods(name: &str, mods: Vec<ModInfo>) {
    let path = profile_path();
    let profiles = get_profile();

    let mut new_profiles: Vec<Profile> = Vec::new();
    for profile in profiles {
        if profile.name == name {
            let new_profile = Profile {
                name: profile.name,
                mods: mods.clone(),
                currently: profile.currently
            };
            new_profiles.push(new_profile);
        } else {
            new_profiles.push(profile);
        }
    }
    save_profiles(new_profiles);
}