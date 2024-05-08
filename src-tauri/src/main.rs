// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{fs};
use std::fs::File;

mod app;
use crate::app::{mods, config, user, profiles};

use tauri::{command, Manager};
use zip::ZipArchive;
use crate::app::{api, game, console, export};
use crate::app::api::downloader;
use crate::app::app_state::AppState;

use crate::app::utility::paths;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[command]
async fn greet(app: tauri::AppHandle) -> String {
    let version = app.package_info().version.clone().to_string();
    format!("<span style='color: #8bc24a'><b>[Junimo]</b> Welcome to Junimo Version {}!</span>", version)
}

#[command]
fn test(handle: tauri::AppHandle, name: &str) -> String {
    let mods = mods::get_all_mods();
    let first_mod = mods.first().unwrap();

    let zip_path = format!("{}.zip", first_mod.name);
    let mut path = paths::mod_path();
    path.push(&zip_path);

    let zip_file = File::open(&path).unwrap();
    let mut zip_archive = ZipArchive::new(zip_file).unwrap();

    // Access the first file in the ZIP archive.
    let first_file = zip_archive.by_index(0).unwrap();

    // Get the file name in a way that ensures it lives long enough.
    let enclosed_name = first_file.enclosed_name().ok_or_else(|| anyhow::anyhow!("Failed to get file name")).unwrap();
    let enclosed_name = enclosed_name.to_string_lossy(); // Cow<str>

    // Now you can safely split and use the parts
    let parts: Vec<&str> = enclosed_name.split('/').collect();

    let first_part = parts.first().unwrap();
    println!("File: {}", first_part);

    "s".to_string()
}

#[command]
async fn show_window(window: tauri::Window, label: String) -> String {
    window.get_webview_window(label.as_str()).unwrap().show().unwrap(); // replace "main" by the name of your window
    "Opened".to_owned()
}

#[command]
async fn close_splashscreen(window: tauri::Window) {
    window.get_webview_window("splashscreen").expect("no window labeled 'splashscreen' found").close().unwrap();
    window.get_webview_window("main").expect("no window labeled 'main' found").show().unwrap();
}

#[derive(Clone, serde::Serialize)]
struct Payload {
    args: Vec<String>,
    cwd: String,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn main() {
    fs::create_dir_all(paths::mod_path()).unwrap();

    let (app_state, rx) = AppState::new();

    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.app_handle();
            config::init_config(&app_handle);

            Ok(())
        })
        .manage(app_state.clone())
        .plugin(tauri_plugin_single_instance::init(move |app, argv, cwd| {
            let stop_signal_clone = app_state.clone();
            let app_handle_clone = app.clone();
            tauri::async_runtime::spawn(async move {
                api::downloader::start_download(&app_handle_clone, argv[1].as_str(), stop_signal_clone).await;
            });
        }))
        .invoke_handler(tauri::generate_handler![
            greet,
            test,
            show_window,
            close_splashscreen,
            game::start_game,
            game::stop_game,
            mods::add_mod,
            mods::get_installed_mods,
            mods::uninstall_mod,
            mods::uninstall_mods,
            mods::open_search_browser,
            config::open_config,
            config::load_config,
            config::select_game_dir,
            config::save_config_button,
            user::connect_user,
            user::disconnect_user,
            user::load_user,
            profiles::get_profiles,
            profiles::get_current_profile,
            profiles::open_profile,
            profiles::change_current_profile,
            profiles::add_profile,
            profiles::remove_profile,
            profiles::modify_profile,
            profiles::change_profile_mods,
            export::open_export,
            export::close_export,
            export::select_export_dir,
            export::export_profile,
            downloader::stop_download,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
