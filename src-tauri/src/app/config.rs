use std::{env, fs};
use std::fs::File;
use std::io::{ Write};
use std::path::{PathBuf};
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use tauri::{command, path, WebviewUrl, Window};
use winreg::enums::*;
use winreg::RegKey;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    init_app: bool,
    pub game_path: String,
    handle_nxm: bool,
}

impl Config {
    fn new(init_app: bool, game_path: String, handle_nxm: bool) -> Config {
        Config { init_app, game_path, handle_nxm }
    }
}

fn config_path() -> PathBuf {
    let mut config_path = dirs::config_dir().unwrap();
    config_path.push("Junimo");
    config_path.push("config.json");
    config_path
}

pub fn get_config() -> Config {
    let path = config_path();
    let data_raw = fs::read_to_string(path).unwrap();
    let data = data_raw.as_str();

    serde_json::from_str(data).unwrap()
}

fn save_config(config: &Config) {
    let json = serde_json::to_string(&config).unwrap();
    let mut file = File::create(config_path()).expect("Fail");
    file.write_all(json.as_bytes()).unwrap();
}

#[command]
pub async fn open_config(handle: tauri::AppHandle) {
    tauri::WebviewWindowBuilder::new(
        &handle,
        "Config",
        WebviewUrl::App("/config".into())
    ).title("Settings").transparent(true).build().unwrap();
}

#[command]
pub fn load_config() -> Config {
    get_config()
}

#[command]
pub fn save_config_button(window: Window, config: Config) {
    save_config(&config);
    if config.handle_nxm {
        register_nxm();
    }
    window.close().expect("Couldn't close window!");
}

#[command]
pub fn select_game_dir() -> String {
    let config = load_config();
    let mut path = ".";
    if config.game_path != "" {
        path = &config.game_path;
    }

    let dir = FileDialog::new()
        .set_directory(path)
        .pick_folder();

    if dir == None {
        return "".to_string();
    }
    dir.unwrap().to_str().unwrap().to_string()
}

pub fn init_config(handle: &tauri::AppHandle) {
    let mut dir_path= dirs::config_dir().unwrap();
    dir_path.push("Junimo");
    fs::create_dir_all(dir_path).unwrap();

    let path = config_path();

    if !path.exists() {
        let config = Config {
            init_app: false,
            game_path: "".to_owned(),
            handle_nxm: false
        };
        save_config(&config);

        tauri::WebviewWindowBuilder::new(
            handle,
            "Config",
            WebviewUrl::App("/config".into())
        ).title("Configuration").build().unwrap();
    }
}

fn register_nxm() {
    let hklm = RegKey::predef(HKEY_CLASSES_ROOT);
    let (nxm, _) = hklm.create_subkey("nxm").unwrap();
    nxm.set_value("", &"URL:NXM Protocol").unwrap();
    nxm.set_value("URL Protocol", &"").unwrap();

    let (command, _) = nxm.create_subkey(r"shell\open\command").unwrap();
    // Specify the path to your application, and `%1` will be replaced by the URL
    let app_path = env::current_exe().unwrap().to_str().unwrap().to_owned() + " \"%1\"";
    command.set_value("", &app_path).unwrap();
}