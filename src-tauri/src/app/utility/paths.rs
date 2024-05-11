use std::fs;
use std::path::PathBuf;
use tauri::command;

pub fn appdata_path() -> PathBuf {
    let mut config_path = dirs::config_dir().unwrap();
    config_path.push("Junimo");
    config_path
}

pub fn mod_path() -> PathBuf {
    let mut mods_path = dirs::config_dir().unwrap();
    mods_path.push("Junimo");
    mods_path.push("mods");
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

#[command]
pub fn config_path() -> PathBuf {
    let mut config_path = dirs::config_dir().unwrap();
    config_path.push("Junimo");
    config_path.push("config.json");
    config_path
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
}
