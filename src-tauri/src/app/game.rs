use crate::app::app_state::AppState;
use crate::app::config::Config;
use crate::app::mods::ModInfo;
use crate::app::utility::{paths, zips};
use crate::app::{config, console, mods, profiles};
use portable_pty::{native_pty_system, Child, CommandBuilder, PtyPair, PtySize};
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use std::{fs, thread};
use sysinfo::System;
use tauri::{command, AppHandle, Manager, State};
use zip::ZipArchive;

#[command]
pub async fn start_game(app_handle: AppHandle, app_state: State<'_, AppState>) -> Result<(), String> {
    console::empty_line(&app_handle);
    console::add_line(
        &app_handle,
        "<span style=\"color: #2fb565\">[Junimo] Starting Stardew Valley via SMAPI</span>"
            .to_string(),
    );

    match init_game(app_handle.clone(), app_state.stop_game.clone()).await {
        Ok(_) => {}
        Err(e) => {
            console::add_line(
                &app_handle,
                format!(
                    "<span style=\"color: #c22f2f\">[Junimo] Failed to start game: {}</span>",
                    e
                ),
            );
        }
    }
    Ok(())
}

async fn init_game(
    original_app_handle: AppHandle,
    original_stop_game: Arc<Mutex<bool>>,
) -> Result<(), String> {
    let app_handle = original_app_handle.clone();
    let stop_game = original_stop_game.clone();

    tokio::spawn(async move {
        let config = config::get_config(paths::config_path());
        let profile = profiles::get_current_profile(original_app_handle, paths::profile_path()).await;
        let mods = profile.mods;
        let mod_path = format!("{}/mods", config.game_path.clone());
        let directories = get_directories(&mod_path).unwrap();

        let mod_names: HashMap<String, String> = mods
            .iter()
            .filter_map(|mod_info| match get_dir_name(mod_info) {
                Ok(dir_name) => Some((dir_name, mod_info.name.clone())),
                Err(_) => None,
            })
            .collect();

        let dirs_to_remove = directories
            .iter()
            .filter(|dir| !mod_names.contains_key(*dir))
            .collect::<Vec<_>>();
        let mods_to_add = mod_names
            .iter()
            .filter_map(|(dir, name)| {
                if !directories.contains(&dir) {
                    Some(mods.iter().find(|mod_info| mod_info.name == *name).unwrap())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        uninstall_mods(&app_handle, &mod_path, dirs_to_remove);
        install_missing_mods(&app_handle, &config, mods_to_add, &stop_game)
            .map_err(|_| {
                console::add_line(
                    &app_handle,
                    "<span style=\"color: #c22f2f\">[Junimo] Failed to install mods</span>"
                        .to_string(),
                );
            })
            .unwrap_or_default();

        let any_missing_mods = mods::any_missing_dependencies(&mods);
        if any_missing_mods {
            console::add_line(&app_handle, "<span style=\"color: #c22f2f\">[Junimo] Missing dependencies detected. Please check your mods.</span>".to_string());
            app_handle.emit("close", true).unwrap();
            return;
        }

        start_smapi(app_handle, &stop_game.clone());
    }).await.unwrap();

    Ok(())
}

fn get_directories<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<String>> {
    let mut directories = Vec::new();
    let entries = fs::read_dir(path)?;

    for entry in entries {
        if let Ok(entry) = entry {
            if entry.path().is_dir() {
                directories.push(entry.file_name().to_string_lossy().to_string());
            }
        }
    }

    Ok(directories)
}

fn get_dir_name(mod_info: &ModInfo) -> Result<String, String> {
    let zip_path = format!("{}.zip", mod_info.name);
    let mut path = paths::mod_path();
    path.push(&zip_path);

    let zip_file = File::open(&path).unwrap();
    let zip_archive = ZipArchive::new(zip_file);
    match zip_archive {
        Ok(mut zip_archive) => {
            let first_file = zip_archive.by_index(0).unwrap();
            let enclosed_name = first_file
                .enclosed_name()
                .ok_or_else(|| anyhow::anyhow!("Failed to get file name"))
                .unwrap();
            let enclosed_name = enclosed_name.to_string_lossy();
            let enclosed_name = enclosed_name.to_string().replace("\\", "/");

            let parts: Vec<&str> = enclosed_name.split('/').collect();

            Ok(parts.first().unwrap().clone().to_string())
        }
        Err(_) => Err("Failed to open zip archive".to_string()),
    }
}

fn uninstall_mods(app_handle: &AppHandle, mod_path: &String, dir_to_remove: Vec<&String>) {
    for dir in dir_to_remove {
        let dir_path = format!("{}/{}", mod_path, dir);
        match fs::remove_dir_all(dir_path) {
            Ok(_) => {
                console::add_line(
                    &app_handle,
                    format!(
                        "<span style=\"color: #c22f2f\">[Junimo] Uninstalled {}</span>",
                        &dir
                    ),
                );
            }
            Err(_) => {
                console::add_line(
                    &app_handle,
                    format!(
                        "<span style=\"color: #c22f2f\">[Junimo] Failed to uninstall {}</span>",
                        &dir
                    ),
                );
            }
        };
    }
}

fn install_missing_mods(
    app_handle: &AppHandle,
    config: &Config,
    mods_to_add: Vec<&ModInfo>,
    stop_game: &Arc<Mutex<bool>>,
) -> Result<(), String> {
    let game_mod_path = format!("{}/Mods", config.game_path.clone());
    let default_mod_path = paths::mod_path();

    for mods in mods_to_add {
        if let Ok(mut lock) = stop_game.lock() {
            if !*lock {
                return Ok(());
            }
        }

        let zip_file_path = format!("{}/{}.zip", &default_mod_path.display(), mods.name);
        let zip_file = File::open(&zip_file_path)
            .map_err(|_| {
                console::add_line(
                    &app_handle,
                    format!(
                        "<span style=\"color: #c22f2f\">[Junimo] Failed to open {}</span>",
                        mods.name
                    ),
                );
            })
            .ok();
        if zip_file.is_none() {
            continue;
        }

        let zip_archive = ZipArchive::new(zip_file.unwrap())
            .map_err(|_| {
                console::add_line(
                    &app_handle,
                    format!(
                        "<span style=\"color: #c22f2f\">[Junimo] Failed to open {}</span>",
                        mods.name
                    ),
                );
            })
            .ok();
        if zip_archive.is_none() {
            continue;
        }

        match zips::extract_zip(zip_archive.unwrap(), &PathBuf::from(&game_mod_path)) {
            Ok(_) => {
                console::add_line(
                    &app_handle,
                    format!(
                        "<span style=\"color: #2fb565\">[Junimo] Installed {}</span>",
                        mods.name
                    ),
                );
            }
            Err(e) => {
                console::add_line(
                    &app_handle,
                    format!(
                        "<span style=\"color: #c22f2f\">[Junimo] Failed to install {}</span>",
                        mods.name
                    ),
                );
            }
        }
    }

    Ok(())
}

fn start_smapi(app_handle: AppHandle, app_state: &Arc<Mutex<bool>>) {
    let stop_signal = app_state.clone();

    #[cfg(target_os = "windows")]
    let game_path = paths::get_game_path().join("StardewModdingAPI.exe");

    #[cfg(not(target_os = "windows"))]
    let game_path = paths::get_game_path().join("StardewModdingAPI.dll");

    let pty_system = native_pty_system();

    let mut pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 2000,
            pixel_width: 0,
            pixel_height: 0,
        })
        .unwrap();

    let cmd = CommandBuilder::new(game_path);
    let mut child = pair.slave.spawn_command(cmd).unwrap();

    read_console(&pair, app_handle.clone());

    thread::spawn(move || loop {
        if set_stop_game(&stop_signal, &child) {
            child.kill().unwrap();
            drop(child);
            drop(pair.slave);
            drop(pair.master);
            app_handle.clone().emit("close", true).unwrap();
            break;
        }
        sleep(Duration::from_millis(200));
    });
}

fn read_console(pair: &PtyPair, app_handle: AppHandle) {
    let reader = pair.master.try_clone_reader().unwrap();
    let mut reader = BufReader::new(reader);
    let mut lines = reader.lines();
    let title_regex = Regex::new(r"\x1b\]0;.*?\x07").unwrap();

    thread::spawn(move || loop {
        let line = lines.next();
        match line {
            Some(Ok(line)) => {
                let clean_line = title_regex.replace_all(&line, "");
                if !clean_line.is_empty() {
                    let html_line = ansi_to_html::convert(&clean_line.to_string()).unwrap();
                    let mut bolded = html_line.replace("[", "<b>[");
                    bolded = bolded.replace("]", "]</b>");
                    console::add_line(&app_handle, bolded);
                }
            }
            Some(Err(_line)) => {
                break;
            }
            None => {
                break;
            }
        }
    });
}

fn set_stop_game(stop_signal: &Arc<Mutex<bool>>, child: &Box<dyn Child + Send + Sync>) -> bool {
    if let Ok(mut lock) = stop_signal.lock() {
        if !*lock {
            *lock = true;
            return true;
        }
    }

    match child.process_id() {
        Some(pid) => {
            let s = System::new_all();
            let child_processes = s
                .processes()
                .values()
                .filter(|proc| proc.pid() == sysinfo::Pid::from_u32(pid))
                .collect::<Vec<_>>();

            if child_processes.len() == 0 {
                return true;
            }
        }
        None => {
            return true;
        }
    }

    false
}

#[command]
pub fn stop_game(app_state: State<'_, AppState>) {
    let mut signal = app_state.stop_game.lock().unwrap();
    *signal = false;
}
