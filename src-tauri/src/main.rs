// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;

use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::{ClickType, TrayIconBuilder};
use tauri::{command, Manager};
use tauri_plugin_updater::UpdaterExt;

use crate::app::api::{downloader, github};
use crate::app::app_state::AppState;
use crate::app::utility::paths;
use crate::app::{api, export, game, smapi, junimo_updater};
use crate::app::{config, import, mods, profiles, user};

mod app;

#[command]
async fn greet(app: tauri::AppHandle) -> String {
    let version = app.package_info().version.clone().to_string();
    format!(
        "<span style='color: #8bc24a'><b>[Junimo]</b> Welcome to Junimo Version {}!</span>",
        version
    )
}

#[command]
async fn init(app_handle: tauri::AppHandle) -> bool {
    mods::compatibility_check(app_handle).await
}

#[command]
async fn show_window(window: tauri::Window, label: String) -> String {
    window
        .get_webview_window(label.as_str())
        .unwrap()
        .show()
        .unwrap();
    "Opened".to_owned()
}

#[command]
async fn close_splashscreen(window: tauri::Window, handle: tauri::AppHandle) {
    if handle.get_webview_window("splashscreen").is_some() {
        handle
            .get_webview_window("splashscreen")
            .expect("no window labeled 'splashscreen' found")
            .close()
            .unwrap();
    }

    handle
        .get_webview_window("main")
        .expect("no window labeled 'main' found")
        .show()
        .unwrap();

    handle
        .get_webview_window("main")
        .expect("no window labeled 'main' found")
        .set_focus()
        .unwrap();
}

#[command]
async fn close(handle: tauri::AppHandle) {
    handle.exit(0);
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
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let app_handle = app.app_handle();
            let _ = config::init_config(&app_handle, paths::appdata_path().as_path());

            let toggle = MenuItemBuilder::with_id("close", "Close").build(app)?;
            let menu = MenuBuilder::new(app).items(&[&toggle]).build()?;
            let tray = TrayIconBuilder::new()
                .menu(&menu)
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "close" => {
                        app.exit(0);
                    }
                    _ => (),
                })
                .on_tray_icon_event(|tray, event| {
                    if event.click_type == ClickType::Left {
                        let app = tray.app_handle();
                        if let Some(webview_window) = app.get_webview_window("main") {
                            let _ = webview_window.show();
                            let _ = webview_window.set_focus();
                        }
                    }
                })
                .icon(app.default_window_icon().cloned().unwrap())
                .build(app)?;

            Ok(())
        })
        .manage(app_state.clone())
        .plugin(tauri_plugin_single_instance::init(move |app, argv, cwd| {
            let stop_signal_clone = app_state.clone();
            let app_handle_clone = app.clone();
            println!("{}", &argv[1].as_str());
            tauri::async_runtime::spawn(async move {
                api::downloader::start_download(
                    &app_handle_clone,
                    argv[1].as_str(),
                    stop_signal_clone,
                )
                .await;
            });
        }))
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                if window.label() == "main" {
                    let config = config::get_config(paths::config_path());
                    if config.keep_open.is_none() || config.keep_open.unwrap() {
                        window.hide().unwrap();
                        api.prevent_close();
                    }
                }
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            init,
            greet,
            show_window,
            close_splashscreen,
            close,
            paths::config_path,
            paths::profile_path,
            game::start_game,
            game::stop_game,
            mods::add_mod,
            mods::get_installed_mods,
            mods::uninstall_mod,
            mods::uninstall_mods,
            mods::open_search_browser,
            config::open_config,
            config::get_config,
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
            profiles::duplicate_profile,
            export::open_export,
            export::select_export_dir,
            export::export_profile,
            import::open_import,
            import::select_import_dir,
            import::import_profile,
            downloader::stop_download,
            github::check_smapi_version,
            smapi::open_smapi,
            smapi::download_smapi,
            junimo_updater::open_updater
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
