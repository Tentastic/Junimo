use std::collections::HashSet;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use std::{env, fs, thread};

use portable_pty::{native_pty_system, Child, CommandBuilder, PtyPair, PtySize};
use regex::Regex;
use sysinfo::System;
use tauri::{command, AppHandle, Manager, State};

use crate::app::app_state::AppState;
use crate::app::models::mod_info::ModInfo;
use crate::app::utility::paths;
use crate::app::{config, console, profiles};

/// Starts the game through the frontend
///
/// * `app_handle` - The app handle
/// * `app_state` - The app state
///
/// # Returns Command result
#[command]
pub async fn start_game(
    app_handle: AppHandle,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    console::empty_line(&app_handle);
    console::add_line(
        &app_handle,
        "<span style=\"color: #2fb565\">[Junimo] Starting Stardew Valley via SMAPI</span>"
            .to_string(),
    );

    match init_game(app_handle.clone(), app_state.stop_game.clone()).await {
        Ok(_) => {}
        Err(e) => {
            app_handle.emit("close", true).unwrap();
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

/// Initializes the game by installing and uninstalling mods and starting SMAPI
///
/// * `original_app_handle` - The app handle
/// * `original_stop_game` - The stop game signal
///
/// # Returns Init game result
async fn init_game(
    original_app_handle: AppHandle,
    original_stop_game: Arc<Mutex<bool>>,
) -> Result<(), String> {
    let app_handle = original_app_handle.clone();
    let stop_game = original_stop_game.clone();

    // Spawn a new thread to initialize the game without blocking the main thread
    let spawn_result = tokio::spawn(async move {
        let config = config::get_config(paths::config_path());
        let profile = profiles::get_current_profile(paths::profile_path()).await;
        let mods = profile.mods;
        let mod_path = paths::mod_path().display().to_string();
        let directories = get_directories(&mod_path).unwrap();

        // Puts all mod names into a HashSet
        let mod_names: HashSet<String> = mods
            .iter()
            .filter_map(|mod_info| Some(mod_info.name.clone()))
            .collect();

        // Puts all directories to remove into a Vec
        let dirs_to_remove = directories
            .iter()
            .filter(|dir| !mod_names.contains(*dir) && !dir.contains("."))
            .collect::<Vec<_>>();

        // Puts all directories to add into a Vec
        let mods_to_add = directories
            .iter()
            .filter(|dir| {
                mod_names.contains(&dir.replace(".", "").to_string()) && dir.contains(".")
            })
            .collect::<Vec<_>>();

        // Uninstall and install mods
        uninstall_mods(&app_handle, &mod_path, dirs_to_remove);
        install_missing_mods(&app_handle, &mod_path, mods_to_add);

        let config = config::get_config(paths::config_path());

        // Check if there are any missing dependencies
        let any_missing_mods = any_missing_dependencies(&mods);
        if any_missing_mods
            && (config.block_on_missing_requirements.is_none()
                || (config.block_on_missing_requirements.is_some()
                    && config.block_on_missing_requirements.unwrap()))
        {
            return Err("Missing requirements detected. Please check your mods.".to_string());
        }

        // Check if there are any broken mods
        let broken_mods = !mods
            .iter()
            .filter(|mod_info| mod_info.is_broken.is_some())
            .collect::<Vec<_>>()
            .is_empty();
        if broken_mods
            && (config.block_on_broken.is_none()
                || (config.block_on_broken.is_some() && config.block_on_broken.unwrap()))
        {
            return Err(
                "Some of your currently installed mods are broken. Please remove or update them."
                    .to_string(),
            );
        }

        let smapi_result = start_smapi(app_handle, &stop_game.clone());
        return match smapi_result {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        };
    })
    .await
    .unwrap();

    return match spawn_result {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    };
}

/// Checks if there are any missing dependencies
///
/// * `mods` - The list of mods to check
///
/// # Returns true if there are any missing dependencies, false otherwise
pub fn any_missing_dependencies(mods: &Vec<ModInfo>) -> bool {
    let cloned_mods = mods.clone();
    let mut new_modinfo = Vec::new();

    for mut mod_info in mods.clone() {
        // If the mod has no dependencies, skip the check and add it to the new_modinfo
        if mod_info.dependencies.is_none() {
            new_modinfo.push(mod_info.clone());
            continue;
        }

        match &mod_info.dependencies {
            Some(dependencies) => {
                for dependency in dependencies {
                    // If the dependency was found, skip the check and add it to the new_modinfo
                    if cloned_mods
                        .iter()
                        .any(|mod_info| mod_info.unique_id == Some(dependency.unique_id.clone()))
                    {
                        continue;
                    }
                    // If the dependency was not found, and it is required, return true
                    if dependency.is_required == None || dependency.is_required == Some(true) {
                        return true;
                    } else {
                        continue;
                    }
                }
                new_modinfo.push(mod_info.clone());
            }
            None => {}
        }
    }
    false
}

/// Gets all mod directories
///
/// * `path` - The path to get the directories from (usually the default Junimo mod path)
///
/// # Returns a vector of strings containing all directories
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

/// Simply sets a point in front of the mod's directory names
///
/// * `app_handle` - The app handle
/// * `mod_path` - The path to the mods (usually the default Junimo mod path)
/// * `dirs_to_remove` - The directories to remove
fn uninstall_mods(app_handle: &AppHandle, mod_path: &String, dir_to_remove: Vec<&String>) {
    for dir in dir_to_remove {
        let dir_path = format!("{}/{}", mod_path, dir);
        let new_path = format!("{}/.{}", mod_path, dir);
        match fs::rename(&dir_path, &new_path) {
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
        }
    }
}

/// Installs missing mods by removing the point in front of the mod's directory names
///
/// * `app_handle` - The app handle
/// * `mod_path` - The path to the mods (usually the default Junimo mod path)
/// * `mods_to_add` - The mods to add
fn install_missing_mods(app_handle: &AppHandle, mod_path: &String, mods_to_add: Vec<&String>) {
    for dir in mods_to_add {
        let dir_path = PathBuf::from(mod_path).join(dir);
        let new_path = PathBuf::from(mod_path).join(dir.replace(".", ""));
        if new_path.exists() {
            fs::remove_dir_all(&new_path).unwrap();
        }
        match fs::rename(&dir_path, &new_path) {
            Ok(_) => {
                console::add_line(
                    &app_handle,
                    format!(
                        "<span style=\"color: #2fb565\">[Junimo] Installed {}</span>",
                        &dir.replace(".", "")
                    ),
                );
            }
            Err(e) => {
                console::add_line(
                    &app_handle,
                    format!(
                        "<span style=\"color: #c22f2f\">[Junimo] Failed to install {} ({})</span>",
                        &dir.replace(".", ""),
                        e.to_string()
                    ),
                );
            }
        }
    }
}

/// This function starts the StardewModdingAPI
///
/// * `app_handle` - The app handle
/// * `app_state` - The app state
fn start_smapi(app_handle: AppHandle, app_state: &Arc<Mutex<bool>>) -> Result<(), String> {
    let stop_signal = app_state.clone();

    // Set the environment variable for the mods path
    let key = "SMAPI_MODS_PATH";
    env::set_var(key, paths::mod_path().display().to_string());

    let game_path = paths::get_game_path();

    if !game_path.exists() || game_path == paths::mod_path() {
        return Err("Game path not found. Please check your settings.".to_string());
    }

    #[cfg(target_os = "windows")]
    if !game_path.clone().join("StardewModdingAPI.exe").exists() {
        return Err(
            "SMAPI was not found! Please install SMAPI before starting the game.".to_string(),
        );
    }
    #[cfg(not(target_os = "windows"))]
    if !game_path.clone().join("StardewModdingAPI.dll").exists() {
        return Err(
            "SMAPI was not found! Please install SMAPI before starting the game.".to_string(),
        );
    }

    // Get the path to the SMAPI executable. On Windows it is a .exe file, on other platforms it is a .dll file
    #[cfg(target_os = "windows")]
    let game_path = game_path.join("StardewModdingAPI.exe");
    #[cfg(not(target_os = "windows"))]
    let game_path = game_path.join("StardewModdingAPI.dll");

    // Open a new PTY
    let pty_system = native_pty_system();

    // Open a new PTY pair
    let mut pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 2000,
            pixel_width: 0,
            pixel_height: 0,
        })
        .unwrap();

    // Spawn the SMAPI executable
    let cmd = CommandBuilder::new(game_path);
    let mut child = pair.slave.spawn_command(cmd).unwrap();

    // Read the console output into Junimo's console
    read_console(&pair, app_handle.clone());

    // Spawn a new thread to check if the game should be stopped.
    // If the game should be stopped, kill the child process and close the PTY
    thread::spawn(move || loop {
        if set_stop_game(&stop_signal, &child) {
            child.kill().unwrap();
            drop(child);
            drop(pair.slave);
            drop(pair.master);
            app_handle.clone().emit("close", true).unwrap();
            break;
        }
        // Sleep for 200ms to reduce CPU usage
        sleep(Duration::from_millis(200));
    });

    Ok(())
}

/// Reads the console output and adds it to Junimo's console
///
/// * `pair` - The PTY pair we created
/// * `app_handle` - The app handle
fn read_console(pair: &PtyPair, app_handle: AppHandle) {
    // Create a new reader and read the console output line by line
    let reader = pair.master.try_clone_reader().unwrap();
    let mut reader = BufReader::new(reader);
    let mut lines = reader.lines();

    // Create a regex to remove the title from the console output
    let title_regex = Regex::new(r"\x1b\]0;.*?\x07").unwrap();

    // Spawn a new thread to read the console output
    thread::spawn(move || loop {
        let line = lines.next();
        // Match the line and add it to Junimo's console if it is not empty or an error
        match line {
            Some(Ok(line)) => {
                // Remove the title from the console output
                let clean_line = title_regex.replace_all(&line, "");

                // Check if the line was not the title and add it to Junimo's console
                if !clean_line.is_empty() {
                    // Convert the ANSI console output to HTML
                    let html_line = ansi_to_html::convert(&clean_line.to_string()).unwrap();

                    // Make anything between square brackets bold
                    let mut bolded = html_line.replace("[", "<b>[");
                    bolded = bolded.replace("]", "]</b>");

                    // Add the line to Junimo's console
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

/// Sets the stop game signal to stop SMAPI and the game
///
/// * `stop_signal` - The stop signal
/// * `child` - The child process
///
/// # Returns true if the game should be stopped, false otherwise
fn set_stop_game(stop_signal: &Arc<Mutex<bool>>, child: &Box<dyn Child + Send + Sync>) -> bool {
    // Lock the stop signal and check if the game should be stopped
    if let Ok(mut lock) = stop_signal.lock() {
        if !*lock {
            *lock = true;
            return true;
        }
    }

    // Check if the child process is still running
    match child.process_id() {
        Some(pid) => {
            // Get the child process by its PID
            let s = System::new_all();
            let child_processes = s
                .processes()
                .values()
                .filter(|proc| proc.pid() == sysinfo::Pid::from_u32(pid))
                .collect::<Vec<_>>();

            // If the child process is not running, return true
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

/// Tauri command to stop the game by setting the stop game signal to true
///
/// * `app_state` - The app state
#[command]
pub fn stop_game(app_state: State<'_, AppState>) {
    let mut signal = app_state.stop_game.lock().unwrap();
    *signal = false;
}
