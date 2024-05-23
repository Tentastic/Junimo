use crate::app::utility::paths;
use crate::app::{config, profiles};
use std::path::PathBuf;
use std::{env, fs};
use tauri::command;

pub fn appdata_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap();
    path.push("Junimo");

    if !path.exists() {
        fs::create_dir_all(&path).unwrap();
    }

    path
}

pub fn mod_path() -> PathBuf {
    let mut mods_path = dirs::config_dir().unwrap();
    mods_path.push("Junimo");
    mods_path.push("mods");

    if !mods_path.exists() {
        fs::create_dir_all(&mods_path).unwrap();
    }

    mods_path
}

pub fn mod_json_path() -> PathBuf {
    let mut mods_path = dirs::config_dir().unwrap();
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

pub fn get_game_path() -> PathBuf {
    let config = config::get_config(config_path());
    let game_path = config.game_path;
    if game_path == "" {
        return appdata_path();
    }
    PathBuf::from(game_path)
}

#[command]
pub fn config_path() -> PathBuf {
    let mut config_path = dirs::config_dir().unwrap();
    config_path.push("Junimo");
    config_path.push("config.json");
    config_path
}

#[command]
pub fn profile_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap();
    path.push("Junimo");
    path.push("profile.json");

    if !path.exists() {
        profiles::save_profiles(&vec![], &path);
    }

    path
}

pub fn get_app_bundle_path() -> Option<String> {
    env::current_exe().ok().and_then(|path| {
        path.parent() // Points to the executable's directory, typically Contents/MacOS
            .and_then(|path| path.parent()) // Go up to Contents
            .and_then(|path| path.parent()) // Go up to the .app bundle
            .map(|path| path.to_string_lossy().into_owned())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_appdata_path() {
        let mut test_dir = dirs::config_dir().unwrap();
        test_dir.push("Junimo");

        let result = appdata_path();
        assert_eq!(result, test_dir);
    }

    #[test]
    fn test_mod_path() {
        let mut test_dir = dirs::config_dir().unwrap();
        test_dir.push("Junimo");
        test_dir.push("mods");

        let result = mod_path();
        assert_eq!(result, test_dir);
    }

    #[test]
    fn test_mod_json_path() {
        let mut test_dir = dirs::config_dir().unwrap();
        test_dir.push("Junimo");
        test_dir.push("mods.json");

        let result = mod_json_path();
        assert_eq!(result, test_dir);
    }

    #[test]
    fn test_temp_path() {
        let mut test_dir = std::env::temp_dir();
        test_dir.push("Junimo");

        let result = temp_path();
        assert_eq!(result, test_dir);
    }

    #[test]
    fn temp_config_path() {
        let mut test_dir = dirs::config_dir().unwrap();
        test_dir.push("Junimo");
        test_dir.push("config.json");

        let result = config_path();
        assert_eq!(result, test_dir);
    }

    #[test]
    fn temp_profile_path() {
        let mut test_dir = dirs::config_dir().unwrap();
        test_dir.push("Junimo");
        test_dir.push("profile.json");

        let result = profile_path();
        assert_eq!(result, test_dir);
    }
}
