use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use tauri::{command, Manager};
use crate::app::mods;
use crate::app::mods::{ModInfo};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Profile {
    pub name: String,
    pub mods: Vec<ModInfo>,
    pub currently: bool
}

pub fn get_profile_path() -> PathBuf {
    let mut mods_path = tauri::api::path::config_dir().unwrap();
    mods_path.push("Junimo");
    mods_path.push("profile.json");
    mods_path
}

pub fn save_profiles(profiles: &Vec<Profile>) {
    let json = serde_json::to_string(&profiles).unwrap();
    let mut file = File::create(get_profile_path()).expect("Fail");
    file.write_all(json.as_bytes()).unwrap();
}

#[command]
pub fn get_profiles() -> Vec<Profile> {
    let path = get_profile_path();

    if !path.exists() {
        let mut profiles: Vec<Profile> = Vec::new();
        let profile = Profile {
            name: "Default".to_string(),
            mods: Vec::new(),
            currently: true
        };
        profiles.push(profile);
        save_profiles(&profiles);
    }

    let data_raw = tauri::api::file::read_string(path).unwrap();
    let data = data_raw.as_str();
    serde_json::from_str(data).unwrap()
}

#[command]
pub fn get_current_profile() -> Profile {
    let path = get_profile_path();

    if !path.exists() {
        let mut profiles: Vec<Profile> = Vec::new();
        let profile = Profile {
            name: "Default".to_string(),
            mods: Vec::new(),
            currently: true
        };
        profiles.push(profile);
        save_profiles(&profiles);
    }

    let data_raw = tauri::api::file::read_string(path).unwrap();
    let data = data_raw.as_str();
    let profiles: Vec<Profile> = serde_json::from_str(data).unwrap();

    let return_profile = profiles.clone();
    for profile in profiles {
        if profile.currently {
            let mut new_profile = profile;
            new_profile.mods = mods::check_dependencies(new_profile.clone().mods);
            return new_profile.clone();
        }
    }

    return_profile[0].clone()
}

#[command]
pub async fn open_profile(handle: tauri::AppHandle) {
    tauri::WindowBuilder::new(
        &handle,
        "Profiles",
        tauri::WindowUrl::App("/profiles".into())
    ).title("Profiles").visible(false).build().unwrap();
}

#[command]
pub fn change_current_profile(handle: tauri::AppHandle, name: &str) -> Vec<Profile> {
    let profiles = get_profiles();

    let mut new_profiles: Vec<Profile> = Vec::new();
    for profile in profiles {
        if profile.name == name {
            let new_profile = Profile {
                name: profile.name,
                mods: profile.mods,
                currently: true
            };
            new_profiles.push(new_profile);
        } else {
            let new_profile = Profile {
                name: profile.name,
                mods: profile.mods,
                currently: false
            };
            new_profiles.push(new_profile);
        }
    }
    handle.emit_all("profile-update", &new_profiles).expect("Failed to emit event");
    save_profiles(&new_profiles);
    new_profiles
}

#[command]
pub fn add_profile(handle: tauri::AppHandle, name: &str) -> Vec<Profile> {
    let profiles = get_profiles();

    let mut new_profiles: Vec<Profile> = Vec::new();
    for profile in profiles {
        let new_profile = Profile {
            name: profile.name,
            mods: profile.mods,
            currently: false
        };
        new_profiles.push(new_profile);
    }
    let new_profile = Profile {
        name: name.to_string(),
        mods: Vec::new(),
        currently: true
    };
    new_profiles.push(new_profile);
    handle.emit_all("profile-update", &new_profiles).expect("Failed to emit event");
    save_profiles(&new_profiles);
    new_profiles
}

#[command]
pub fn remove_profile(handle: tauri::AppHandle, name: &str) -> Vec<Profile> {
    let profiles = get_profiles();

    let mut new_profiles: Vec<Profile> = Vec::new();
    for profile in profiles {
        if profile.name != name {
            new_profiles.push(profile);
        }
    }
    handle.emit_all("profile-update", &new_profiles).expect("Failed to emit event");
    save_profiles(&new_profiles);
    new_profiles
}

#[command]
pub fn modify_profile(handle: tauri::AppHandle, name: &str, new_name: &str) -> Vec<Profile> {
    let profiles = get_profiles();

    let mut new_profiles: Vec<Profile> = Vec::new();
    for profile in profiles {
        if profile.name == name {
            let new_profile = Profile {
                name: new_name.parse().unwrap(),
                mods: profile.mods,
                currently: profile.currently
            };
            new_profiles.push(new_profile);
        } else {
            new_profiles.push(profile);
        }
    }
    handle.emit_all("profile-update", &new_profiles).expect("Failed to emit event");
    save_profiles(&new_profiles);
    new_profiles
}

#[command]
pub fn change_profile_mods(handle: tauri::AppHandle, name: &str, mods: Vec<ModInfo>) {
    let profiles = get_profiles();

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
    save_profiles(&new_profiles);
}