use std::path::PathBuf;

pub fn app_path(file: &str) -> PathBuf {
    let mut path = tauri::api::path::config_dir().unwrap();
    path.push("Junimo");
    path.push(file);
    path
}