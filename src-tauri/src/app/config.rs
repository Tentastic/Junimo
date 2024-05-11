use std::{env, fs};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{command, Manager, Runtime, WebviewUrl, WebviewWindow, Window};
use winreg::enums::*;
use winreg::RegKey;

use crate::app::app_state::AppState;
use crate::app::console;
use crate::app::utility::paths;
use crate::testing::register_manager::{RealRegistryManager, RegistryManager};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    init_app: bool,
    pub game_path: String,
    handle_nxm: bool,
}

impl Config {
    fn new(init_app: bool, game_path: String, handle_nxm: bool) -> Config {
        Config {
            init_app,
            game_path,
            handle_nxm,
        }
    }
}

#[command]
pub fn get_config(path: PathBuf) -> Config {
    let data_raw = fs::read_to_string(path).unwrap();
    let data = data_raw.as_str();
    serde_json::from_str(data).unwrap()
}

fn save_config(config: &Config, path: &PathBuf) {
    let json = serde_json::to_string(&config).unwrap();
    let mut file = File::create(path).expect("Fail");
    file.write_all(json.as_bytes()).unwrap();
}

#[command]
pub async fn open_config<R: Runtime>(handle: tauri::AppHandle<R>) {
    tauri::WebviewWindowBuilder::new(&handle, "Config", WebviewUrl::App("/config".into()))
        .title("Settings")
        .transparent(true)
        .build()
        .unwrap();
}

#[command]
pub fn save_config_button<R: Runtime>(window: Window<R>, config: Config, path: PathBuf) {
    save_config(&config, &path);
    if config.handle_nxm {
        let real_register = RealRegistryManager::new().unwrap();
        let _nxm_result = register_nxm(&real_register);
    }
    window.close().expect("Couldn't close window!");
}

#[command]
pub fn select_game_dir(state: tauri::State<AppState>, path: PathBuf) -> String {
    let config = get_config(path);
    let mut path = ".";
    if config.game_path != "" {
        path = &config.game_path;
    }

    let lock = state.directory_selector.lock().unwrap();
    match lock.pick_directory(path) {
        Some(dir) => dir.to_str().unwrap_or_default().to_string(),
        None => "".to_string(),
    }
}

pub fn init_config<R: Runtime>(handle: &tauri::AppHandle<R>, dir: &Path) -> Result<Option<WebviewWindow<R>>, String> {
    let mut path = dir.to_owned();
    path.push("Junimo");
    fs::create_dir_all(&path).unwrap();
    path.push("config.json");

    if !path.exists() {
        let config = Config {
            init_app: false,
            game_path: "".to_owned(),
            handle_nxm: false,
        };
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

fn register_nxm(registry: &dyn RegistryManager) -> Result<(), String> {
    let nxm = registry.create_subkey("nxm").unwrap();
    nxm.set_value("", "URL:NXM Protocol").unwrap();
    nxm.set_value("URL Protocol", "").unwrap();

    let command = nxm.create_subkey(r"shell\open\command").unwrap();
    let app_path = env::current_exe().unwrap().to_str().unwrap().to_owned() + " \"%1\"";
    command.set_value("", &app_path).unwrap();
    Ok(())
}


#[cfg(test)]
mod tests {
    use tauri::test::mock_builder;
    use tempfile::tempdir;

    use crate::testing::directory_selector::MockDirectorySelector;
    use crate::testing::register_manager::MockRegistryManager;

    use super::*;

    fn create_app<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::App<R> {
        let (app_state, rx) = AppState::new(Box::new(MockDirectorySelector));

        builder
            .invoke_handler(tauri::generate_handler![open_config, save_config_button, select_game_dir])
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
    fn test_new_config_0() {
        let expected_config = Config {
            init_app: false,
            game_path: "".to_string(),
            handle_nxm: false,
        };
        let result = Config::new(false, "".to_string(), false);
        assert_eq!(result.init_app, expected_config.init_app);
    }

    #[test]
    fn test_get_config_1() {
        let tmp_dir = tempdir().unwrap();
        let config_path = tmp_dir.path().join("config.json");
        let expected_config = Config {
            init_app: false,
            game_path: "".to_string(),
            handle_nxm: false,
        };
        let serialized_payload = serde_json::to_string(&expected_config).unwrap();
        let mut file = File::create(&config_path).expect("Fail");
        writeln!(file, "{}", serialized_payload).expect("Couldn't write");

        let result = get_config(config_path.clone());
        assert_eq!(result.init_app, expected_config.init_app);
    }

    #[test]
    fn test_save_config_2() {
        let tmp_dir = tempdir().unwrap();
        let config_path = tmp_dir.path().join("config.json");

        let config = Config {
            init_app: false,
            game_path: "".to_string(),
            handle_nxm: false,
        };

        save_config(&config, &config_path);

        assert_eq!(config_path.exists(), true);
    }

    #[test]
    fn test_open_config_3() {
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
    fn test_save_config_button_4() {
        let app = create_app(mock_builder());
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();

        let tmp_dir = tempdir().unwrap();
        let config_path = tmp_dir.path().join("config.json");

        let expected_config = Config {
            init_app: false,
            game_path: "".to_string(),
            handle_nxm: true,
        };
        let config_wrap = ConfigWrap {
            config: expected_config,
            path: config_path
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
    fn test_select_game_dir_5() {
        let app = create_app(mock_builder());
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();

        let tmp_dir = tempdir().unwrap();
        let config_path = tmp_dir.path().join("config.json");

        let config = Config {
            init_app: false,
            game_path: "/test".to_string(),
            handle_nxm: false,
        };
        save_config(&config, &config_path);

        let config_wrap = PathWrap {
            path: config_path
        };
        let serialized_payload = serde_json::to_string(&config_wrap).unwrap();
        let invoke_body = tauri::ipc::InvokeBody::Json(serialized_payload.parse().unwrap());

        let res = tauri::test::get_ipc_response(
            &webview,
            tauri::webview::InvokeRequest {
                cmd: "select_game_dir".into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: "http://tauri.localhost".parse().unwrap(),
                body: invoke_body,
                headers: Default::default(),
            },
        );
        assert!(res.is_ok());
        assert_eq!(res.unwrap().deserialize::<String>().unwrap(), "/test")
    }

    #[test]
    fn test_select_game_dir_none_5() {
        let app = create_app(mock_builder());
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();

        let tmp_dir = tempdir().unwrap();
        let config_path = tmp_dir.path().join("config.json");

        let config = Config {
            init_app: false,
            game_path: " ".to_string(),
            handle_nxm: false,
        };
        save_config(&config, &config_path);

        let config_wrap = PathWrap {
            path: config_path
        };
        let serialized_payload = serde_json::to_string(&config_wrap).unwrap();
        let invoke_body = tauri::ipc::InvokeBody::Json(serialized_payload.parse().unwrap());

        let res = tauri::test::get_ipc_response(
            &webview,
            tauri::webview::InvokeRequest {
                cmd: "select_game_dir".into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: "http://tauri.localhost".parse().unwrap(),
                body: invoke_body,
                headers: Default::default(),
            },
        );
        assert!(res.is_ok());
        assert_eq!(res.unwrap().deserialize::<String>().unwrap(), "")
    }

    #[test]
    fn test_load_config_6() {
        let tmp_dir = tempdir().unwrap();
        let config_path = tmp_dir.path().join("config.json");

        let config = Config {
            init_app: false,
            game_path: "".to_string(),
            handle_nxm: false,
        };

        save_config(&config, &config_path);

        let serialized_payload = serde_json::to_string(&config).unwrap();
        let mut file = File::create(&config_path).expect("Fail");
        writeln!(file, "{}", serialized_payload).expect("Couldn't write");

        let result = get_config(config_path.clone());
        assert_eq!(result.init_app, config.init_app);
    }

    #[test]
    fn test_init_config_7() {
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

    #[test]
    fn test_register_nxm_8() {
        let mock_registry = MockRegistryManager;
        let nxm_result = register_nxm(&mock_registry);
        assert!(nxm_result.is_ok());
    }
}
