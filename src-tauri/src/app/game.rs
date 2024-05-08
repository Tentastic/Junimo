use tauri::{AppHandle, command, Manager, State};
use std::io::{BufReader, BufRead};
use std::{fs, thread};
use crate::app::{config, console, mods, profiles};
use regex::Regex;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::Duration;
use portable_pty::{CommandBuilder, native_pty_system, PtySize};
use crate::app::app_state::AppState;
use sysinfo::{System};
use zip::ZipArchive;
use crate::app::config::Config;
use crate::app::mods::{ModInfo};
use crate::app::utility::{paths, zips};

fn get_dir_name(mod_info: &ModInfo) -> String {
    let zip_path = format!("{}.zip", mod_info.name);
    let mut path = paths::mod_path();
    path.push(&zip_path);

    let zip_file = File::open(&path).unwrap();
    let mut zip_archive = ZipArchive::new(zip_file).unwrap();
    let first_file = zip_archive.by_index(0).unwrap();

    let enclosed_name = first_file.enclosed_name().ok_or_else(|| anyhow::anyhow!("Failed to get file name")).unwrap();
    let enclosed_name = enclosed_name.to_string_lossy();
    let enclosed_name = enclosed_name.to_string().replace("\\", "/");

    let parts: Vec<&str> = enclosed_name.split('/').collect();

    parts.first().unwrap().clone().to_string()
}

fn get_directories<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<String>> {
    let mut directories = Vec::new();
    let entries = fs::read_dir(path)?;

    for entry in entries {
        let entry = entry?;
        if entry.path().is_dir() {
            directories.push(entry.file_name().to_string_lossy().to_string());
        }
    }

    Ok(directories)
}

#[command]
pub fn start_game(app_handle: AppHandle, app_state: State<'_, AppState>) {
    console::empty_line(&app_handle);
    console::add_line(&app_handle, "<span style=\"color: #2fb565\">[Junimo] Starting Stardew Valley via SMAPI</span>".to_string());

    let config = config::load_config();
    let profile = profiles::get_current_profile();
    let mods = profile.mods;

    let mut mods_to_add: Vec<ModInfo> = Vec::new();
    let mut dir_to_remove: Vec<String> = Vec::new();

    let mod_path = format!("{}/Mods", config.game_path.clone());
    let directories = get_directories(&mod_path).unwrap();

    for mod_info in &mods {
        console::add_line(&app_handle, format!("<span style=\"color: #79418a\">[Junimo] Loading infos for {} ({})</span>", &mod_info.name, &mod_info.version));
        let dir_name = get_dir_name(&mod_info);
        if directories.contains(&dir_name) {
            continue;
        }
        mods_to_add.push(mod_info.clone());
    }

    for directory in directories {
        if mods.iter().any(|mod_info| get_dir_name(&mod_info) == directory) {
            continue;
        }
        dir_to_remove.push(directory);
    }

    uninstall_mods(&app_handle, &mod_path, dir_to_remove);
    install_missing_mods(&app_handle, &config, mods_to_add);

    let any_missing_mods = mods::any_missing_dependencies(&mods);
    if any_missing_mods {
        console::add_line(&app_handle, "<span style=\"color: #c22f2f\">[Junimo] Missing dependencies detected. Please check your mods.</span>".to_string());
        app_handle.emit_all("close", true).unwrap();
        return;
    }

    start_smapi(app_handle, app_state.clone());
}

fn uninstall_mods(app_handle: &AppHandle, mod_path: &String, dir_to_remove: Vec<String>) {
    for dir in dir_to_remove {
        console::add_line(&app_handle, format!("<span style=\"color: #c22f2f\">[Junimo] Uninstalling {}</span>", &dir));
        let dir_path = format!("{}/{}", mod_path, dir);
        fs::remove_dir_all(dir_path).unwrap();
    }
}

fn install_missing_mods(app_handle: &AppHandle, config: &Config, mods_to_add: Vec<ModInfo>) {
    let mod_path = format!("{}/Mods", config.game_path.clone());
    let default_mod_path = paths::mod_path();

    for mods in mods_to_add {
        console::add_line(&app_handle, format!("<span style=\"color: #2fb565\">[Junimo] Installing {}</span>", mods.name));

        let zip_file_path = format!("{}/{}.zip", &default_mod_path.display(), mods.name);
        let zip_file = File::open(&zip_file_path).unwrap();
        let zip_archive = ZipArchive::new(zip_file).unwrap();

        println!("{}", &zip_file_path.to_string());
        if !Path::new(&zip_file_path).exists() {
            println!("Path doesn't exists");
        }

        zips::extract_zip(zip_archive, &PathBuf::from(&mod_path)).unwrap();
    }
}

fn start_smapi(app_handle: AppHandle, app_state: State<'_, AppState>) {
    let stop_signal = app_state.stop_game.clone();

    let config = config::get_config();
    let mut game_path = config.game_path;
    game_path.push_str("/StardewModdingAPI.exe");

    let pty_system = native_pty_system();

    let mut pair = pty_system.openpty(PtySize {
        rows: 24,
        cols: 2000,
        pixel_width: 0,
        pixel_height: 0,
    }).unwrap();

    let cmd = CommandBuilder::new(game_path);
    let mut child = pair.slave.spawn_command(cmd).unwrap();

    let reader = pair.master.try_clone_reader().unwrap();
    let mut reader = BufReader::new(reader);
    let mut lines = reader.lines();
    let title_regex = Regex::new(r"\x1b\]0;.*?\x07").unwrap();

    let app_handle_copy = app_handle.clone();
    thread::spawn(move || {
        loop {
            let line = lines.next();
            match line {
                Some(Ok(line)) => {
                    let clean_line = title_regex.replace_all(&line, "");
                    //let clean_line = &line;
                    if !clean_line.is_empty() {
                        let html_line = ansi_to_html::convert(&clean_line.to_string()).unwrap();
                        let mut bolded = html_line.replace("[", "<b>[");
                        bolded = bolded.replace("]", "]</b>");
                        console::add_line(&app_handle, bolded);
                    }
                },
                Some(Err(_line)) => {
                    break;
                },
                None => {
                    break;
                },
            }
        }
    });

    thread::spawn(move || {
        loop {
            if let Ok(mut lock) = stop_signal.lock() {
                if !*lock {
                    child.kill().unwrap();
                    drop(child);
                    drop(pair.slave);
                    drop(pair.master);
                    *lock = true;
                    break;
                }
            }

            match child.process_id() {
                Some(pid) => {
                    let s = System::new_all();
                    let child_processes = s.processes()
                        .values()
                        .filter(|proc| proc.pid() == sysinfo::Pid::from_u32(pid))
                        .collect::<Vec<_>>();

                    if child_processes.len() == 0 {
                        child.kill().unwrap();
                        drop(child);
                        drop(pair.slave);
                        drop(pair.master);
                        app_handle_copy.emit_all("close", true).unwrap();
                        break;
                    }
                },

                None => {
                    child.kill().unwrap();
                    drop(child);
                    drop(pair.slave);
                    drop(pair.master);
                    break;
                }
            }
            sleep(Duration::from_millis(50));
        }
    });
}

#[command]
pub fn stop_game(app_state: State<'_, AppState>) {
    let mut signal = app_state.stop_game.lock().unwrap();
    *signal = false;
}