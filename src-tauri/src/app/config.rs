use std::fs::File;
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};
use std::{env, fs};

use serde::{Deserialize, Serialize};
use tauri::{command, Manager, Runtime, WebviewUrl, WebviewWindow, Window};

#[cfg(target_os = "windows")]
use winreg::enums::HKEY_CLASSES_ROOT;
#[cfg(target_os = "windows")]
use winreg::RegKey;
use crate::app::api::nexuswebsocket;

use crate::app::app_state::AppState;
use crate::app::utility::paths;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    init_app: bool,
    pub game_path: String,
    handle_nxm: bool,
    pub activate_requirements: Option<bool>,
    pub block_on_missing_requirements: Option<bool>,
    pub activate_broken: Option<bool>,
    pub block_on_broken: Option<bool>,
    pub lang: Option<String>,
    pub keep_open: Option<bool>,
}

impl Config {
    fn new(init_app: bool, game_path: String, handle_nxm: bool) -> Config {
        Config {
            init_app,
            game_path,
            handle_nxm,
            activate_requirements: Some(true),
            block_on_missing_requirements: Some(true),
            activate_broken: Some(true),
            block_on_broken: Some(true),
            lang: Some("en".to_string()),
            keep_open: Some(true),
        }
    }
}

/// Gets config from config.json file in appdata
#[command]
pub fn get_config(path: PathBuf) -> Config {
    let data_raw = fs::read_to_string(path).unwrap();
    let data = data_raw.as_str();
    serde_json::from_str(data).unwrap()
}

/// Saves config into config.json file
fn save_config(config: &Config, path: &PathBuf) {
    let json = serde_json::to_string(&config).unwrap();
    let mut file = File::create(path).expect("Fail");
    file.write_all(json.as_bytes()).unwrap();
}

/// Open config window
#[command]
pub async fn open_config<R: Runtime>(handle: tauri::AppHandle<R>) {
    #[cfg(target_os = "windows")]
    tauri::WebviewWindowBuilder::new(&handle, "Config", WebviewUrl::App("/config".into()))
        .title("Settings")
        .transparent(true)
        .build()
        .unwrap();

    #[cfg(target_os = "unix")]
    tauri::WebviewWindowBuilder::new(&handle, "Config", WebviewUrl::App("/config".into()))
        .title("Settings")
        .build()
        .unwrap();
}

#[command]
pub fn save_config_button<R: Runtime>(
    window: Window<R>,
    handle: tauri::AppHandle<R>,
    config: Config,
    path: PathBuf,
) {
    let old_config = get_config(paths::config_path());

    if &old_config.lang != &config.lang {
        &handle.emit("language_changed", &config.lang).unwrap();
    }

    save_config(&config, &path);
    if config.handle_nxm {
        let _nxm_result = register_nxm();
    }
    window.close().expect("Couldn't close window!");
}

pub fn init_config<R: Runtime>(
    handle: &tauri::AppHandle<R>,
    dir: &Path,
) -> Result<Option<WebviewWindow<R>>, String> {
    let mut path = dir.to_owned();
    fs::create_dir_all(&path).unwrap();
    path.push("config.json");

    if !path.exists() {
        let config = Config::new(false, "".to_string(), false);
        save_config(&config, &path);

        let window_result =
            tauri::WebviewWindowBuilder::new(handle, "Config", WebviewUrl::App("/config".into()))
                .title("Configuration")
                .build()
                .unwrap();
        return Ok(Some(window_result));
    }

    Ok(None)
}

#[command]
pub fn load_api_key() -> String {
    nexuswebsocket::load_key()
}

#[command]
pub fn set_api_key(key: String) {
    nexuswebsocket::save_key(key);
}

#[cfg(target_os = "windows")]
fn register_nxm() -> Result<(), String> {
    let hklm = RegKey::predef(HKEY_CLASSES_ROOT);
    let (nxm, _) = hklm.create_subkey("nxm").unwrap();
    nxm.set_value("", &"URL:NXM Protocol").unwrap();
    nxm.set_value("URL Protocol", &"").unwrap();

    let (command, _) = nxm.create_subkey(r"shell\open\command").unwrap();
    let app_path = env::current_exe().unwrap().to_str().unwrap().to_owned() + " \"%1\"";
    command.set_value("", &app_path).unwrap();
    Ok(())
}

#[cfg(target_os = "linux")]
fn register_nxm() -> Result<(), String> {
    let home_dir = env::var("HOME").map_err(|_| "Failed to get HOME directory".to_string())?;
    let local_applications_path = PathBuf::from(home_dir).join(".local/share/applications");

    fs::create_dir_all(&local_applications_path).map_err(|e| e.to_string())?;

    let desktop_entry_path = local_applications_path.join("nxm-handler.desktop");
    let mut file = fs::File::create(&desktop_entry_path).map_err(|e| e.to_string())?;

    let current_exe = env::current_exe().map_err(|e| e.to_string())?;
    let exec_path = current_exe
        .to_str()
        .ok_or("Executable path is invalid UTF-8")?;

    let contents = format!(
        "[Desktop Entry]\nName=NXM Handler\nExec={} \"%u\"\nType=Application\nNoDisplay=true\nMimeType=x-scheme-handler/nxm;\n",
        exec_path
    );
    file.write_all(contents.as_bytes())
        .map_err(|e| e.to_string())?;

    // Updating the MIME-type database
    let output = std::process::Command::new("update-desktop-database")
        .arg(local_applications_path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err("Failed to update desktop database".to_string());
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn register_nxm() -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use tauri::test::mock_builder;
    use tempfile::tempdir;

    use super::*;

    fn create_app<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::App<R> {
        let (app_state, rx) = AppState::new();

        builder
            .invoke_handler(tauri::generate_handler![open_config, save_config_button])
            .manage(app_state.clone())
            // remove the string argument to use your app's config file
            .build(tauri::generate_context!())
            .expect("failed to build app")
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct ConfigWrap {
        config: Config,
        path: PathBuf,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct PathWrap {
        path: PathBuf,
    }

    #[test]
    fn test_new_config() {
        let config = Config::new(false, "".to_string(), false);
        let result = Config::new(false, "".to_string(), false);
        assert_eq!(result.init_app, config.init_app);
    }

    #[test]
    fn test_get_config() {
        let tmp_dir = tempdir().unwrap();
        let config_path = tmp_dir.path().join("config.json");
        let config = Config::new(false, "".to_string(), false);
        let expected_config = Config::new(false, "".to_string(), false);
        let serialized_payload = serde_json::to_string(&expected_config).unwrap();
        let mut file = File::create(&config_path).expect("Fail");
        writeln!(file, "{}", serialized_payload).expect("Couldn't write");

        let result = get_config(config_path.clone());
        assert_eq!(result.init_app, expected_config.init_app);
    }

    #[test]
    fn test_save_config() {
        let tmp_dir = tempdir().unwrap();
        let config_path = tmp_dir.path().join("config.json");

        let config = Config::new(false, "".to_string(), false);

        save_config(&config, &config_path);

        assert_eq!(config_path.exists(), true);
    }

    #[test]
    fn test_open_config() {
        let app = create_app(mock_builder());
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();

        let res = tauri::test::get_ipc_response(
            &webview,
            tauri::webview::InvokeRequest {
                cmd: "open_config".into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: "http://tauri.localhost".parse().unwrap(),
                body: tauri::ipc::InvokeBody::default(),
                headers: Default::default(),
            },
        );
        assert!(res.is_ok());
    }

    #[test]
    fn test_save_config_button() {
        let app = create_app(mock_builder());
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();

        let tmp_dir = tempdir().unwrap();
        let config_path = tmp_dir.path().join("config.json");

        let config = Config::new(false, "".to_string(), false);
        let config_wrap = ConfigWrap {
            config,
            path: config_path,
        };
        let serialized_payload = serde_json::to_string(&config_wrap).unwrap();
        let invoke_body = tauri::ipc::InvokeBody::Json(serialized_payload.parse().unwrap());

        let res = tauri::test::get_ipc_response(
            &webview,
            tauri::webview::InvokeRequest {
                cmd: "save_config_button".into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: "http://tauri.localhost".parse().unwrap(),
                body: invoke_body,
                headers: Default::default(),
            },
        );

        assert!(res.is_ok(), "Saved config button failed");
    }

    #[test]
    fn test_load_config() {
        let tmp_dir = tempdir().unwrap();
        let config_path = tmp_dir.path().join("config.json");

        let config = Config::new(false, "".to_string(), false);

        save_config(&config, &config_path);

        let serialized_payload = serde_json::to_string(&config).unwrap();
        let mut file = File::create(&config_path).expect("Fail");
        writeln!(file, "{}", serialized_payload).expect("Couldn't write");

        let result = get_config(config_path.clone());
        assert_eq!(result.init_app, config.init_app);
    }

    #[test]
    fn test_init_config() {
        let tauri_app = tauri::test::mock_app();
        let handle = tauri_app.handle().clone();

        let temp_dir = tempdir().unwrap();
        let first_result = init_config(&handle, &temp_dir.path());

        let config_path = temp_dir.path().join("Junimo").join("config.json");
        assert!(
            config_path.exists(),
            "Config file should exist after initialization"
        );
        assert!(
            first_result.is_ok_and(|x| x != None),
            "Window should be created if config file doesn't exist"
        );

        let second_result = init_config(&handle, &temp_dir.path());
        assert!(
            second_result.is_ok_and(|x| x == None),
            "Second init should not create window"
        );
    }
}
