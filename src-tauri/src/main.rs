// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;

use std::fs;
use std::sync::{Mutex, Arc};
use app::{mods, config, user, profiles};

use tauri::{Manager};
use tauri::async_runtime::handle;
use crate::app::{api, game};
use crate::app::app_state::AppState;

use crate::app::config::init_config;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
async fn greet(handle: tauri::AppHandle) {
    tauri::WindowBuilder::new(
        &handle,
        "external", /* the unique window label */
        tauri::WindowUrl::App("/test".into())
      ).build().unwrap();
}

#[tauri::command]
fn test(handle: tauri::AppHandle, name: &str) -> String {
    let mut download: api::downloader::Download = api::downloader::Download {
        name: name.to_string(),
        size: 12231,
        current: 0
    };

    handle.emit_all("console", "<span class=\"console-green\">JUNGE</span> MACH").unwrap();
    handle.emit_all("download", download).unwrap();

    name.to_string()
}

#[tauri::command]
fn handle_new_invocation(app_handle: tauri::AppHandle, message: String) {
    println!("New invocation with message: {}", message);
    // Process the message as needed
}

#[derive(Clone, serde::Serialize)]
struct Payload {
    args: Vec<String>,
    cwd: String,
}

fn main() {
    fs::create_dir_all(mods::get_path()).unwrap();

    let (app_state, rx) = AppState::new();

    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.app_handle();
            app_handle.emit_all("console", "Started Junimo Mod Client Version 1.0.0").unwrap();

            config::init_config(&app_handle);

            Ok(())
        })
        .manage(app_state.clone())
        .plugin(tauri_plugin_single_instance::init(move |app, argv, cwd| {
            println!("{}, {argv:?}, {cwd}", app.package_info().name);
            let stop_signal_clone = app_state.clone();
            let app_handle_clone = app.clone();
            tauri::async_runtime::spawn(async move {
                api::downloader::start_download(&app_handle_clone, argv[1].as_str(), stop_signal_clone).await;
            });
        }))

        .invoke_handler(tauri::generate_handler![
            greet,
            test,
            game::start_game,
            game::stop_game,
            mods::add_mod,
            mods::get_mods,
            mods::get_all_mods,
            config::open_config,
            config::load_config,
            config::select_game_dir,
            config::save_config_button,
            user::connect_user,
            user::load_user,
            profiles::get_profile,
            profiles::change_profile_mods,
            api::downloader::stop_download,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
