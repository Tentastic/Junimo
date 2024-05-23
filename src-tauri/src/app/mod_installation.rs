use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::{fs, io};

use tauri::{AppHandle, Manager};
use walkdir::WalkDir;
use zip::ZipArchive;

use crate::app::api::compatibility;
use crate::app::models::mod_info::ModInfo;
use crate::app::mods::{get_all_mods, save_mods, Dependency, Manifest};
use crate::app::utility::{paths, zips};
use crate::app::{config, console};

/// Start the installation of a mod
///
/// * `app_handle` - The handle to the Tauri app
/// * `path` - The path to the zip of the mod file
///
/// # Returns a Result with the success state
pub async fn start_installation(app_handle: AppHandle, path: &PathBuf) -> Result<(), String> {
    let cloned_handle = app_handle.clone();
    let path_clone = path.clone();
    // Start the installation in a new thread to prevent the UI from freezing
    tokio::spawn(async move {
        console::add_line(
            &app_handle,
            "<span class=\"console-green\">[Junimo] Started to add mod</span>".to_string(),
        );

        let zip_file_path = path_clone;
        let output_folder_path = paths::temp_path();
        let to_path = format!(
            "{}/{}",
            output_folder_path.display(),
            zip_file_path.file_name().unwrap().to_str().unwrap()
        );

        if zip_file_path
            .clone()
            .to_string_lossy()
            .to_string()
            .replace("\\", "/")
            != PathBuf::from(&to_path)
                .to_string_lossy()
                .to_string()
                .replace("\\", "/")
        {
            fs::copy(&zip_file_path, &to_path).unwrap();
        }
        let copied_zip_file = File::open(&to_path).unwrap();
        let zip_archive = ZipArchive::new(copied_zip_file).unwrap();

        extract_mod(&cloned_handle, zip_archive, &output_folder_path).await;
    })
    .await
    .unwrap();

    Ok(())
}

/// Extracts the mod from the zip file into the temp folder
///
/// * `app_handle` - The handle to the Tauri app
/// * `archive` - The mods zip archive to unpack
/// * `destination` - The destination path to unpack the mod to
async fn extract_mod<R: io::Read + io::Seek>(
    app_handle: &AppHandle,
    mut archive: ZipArchive<R>,
    destination: &Path,
) {
    let mut main_dir = "".to_string();
    let max = archive.len();
    let mut depth = 0;

    console::add_line(&app_handle, install_progress(&0, &max).to_string());

    // Unpack mods zip file
    match zips::unpack_zip(app_handle, archive, destination, max) {
        Ok(main) => {
            main_dir = main;
        }
        Err(error) => {
            console::modify_line(
                &app_handle,
                format!(
                    "<span class=\"console-red\">[Junimo] Failed to install mod: {}</span>",
                    error
                ),
            );
            &app_handle.emit("reload", false).unwrap();
            return;
        }
    }

    let outpath = destination.join(main_dir);
    let walkdir = WalkDir::new(&outpath);
    let it = walkdir.into_iter();
    let mut group_name = None;

    // Go through unpacked zip and find the manifest.json file. If file depth is over 3 then it's a group
    for entry in it.filter_map(|e| e.ok()) {
        if entry.file_name().to_string_lossy() == "manifest.json" {
            let replaced = entry.path().to_string_lossy().replace("\\", "/");
            let split = replaced.split('/').collect::<Vec<&str>>();
            let temp_path_depth = paths::temp_path()
                .to_string_lossy()
                .replace("\\", "/")
                .split("/")
                .collect::<Vec<&str>>()
                .len();

            // Calculate the depth of the current path
            let temp_depth = split.clone().len() - temp_path_depth.clone();

            // If the depth is 0, set the depth to the current depth.
            // Else if the depth is higher than the current depth, set the depth to the current depth and remove group name.
            if &depth == &0 {
                depth = temp_depth;
            } else if &depth > &temp_depth {
                depth = temp_depth;
                group_name = None;
                continue;
            }

            if &depth < &3 {
                continue;
            }
            group_name = Some(split[temp_path_depth + 1].to_string());
            continue;
        }
    }

    let mut directories: Vec<PathBuf> = Vec::new();

    // If depth is more than 3 extract all directories in the unpacked zip
    if depth < 3 {
        let dir = outpath.clone();
        directories = vec![dir];
    } else {
        directories = list_dirs_in_directory(&outpath);
    }

    match install_mods(&directories, &outpath, &depth, group_name).await {
        Ok(_) => {}
        Err(error) => {
            console::modify_line(
                &app_handle,
                format!(
                    "<span class=\"console-red\">[Junimo] Failed to install mod: {}</span>",
                    error
                ),
            );
            &app_handle.emit("reload", false).unwrap();
            return;
        }
    }

    console::modify_line(
        &app_handle,
        format!("<span class=\"console-green\">[Junimo] Mod installed</span>"),
    );
    &app_handle.emit("reload", true).unwrap();
}

/// Starts the installation of the mod
///
/// * `directories` - The directories of the mod that should be installed
/// * `outpath` - The path to install the mod to
/// * `depth` - The depth of the current mod
/// * `group_name` - The name of the group. None if it's not a group
///
/// # Returns a Result with the success state
async fn install_mods(
    directories: &Vec<PathBuf>,
    outpath: &PathBuf,
    depth: &usize,
    group_name: Option<String>,
) -> Result<(), String> {
    let mut manifest: Option<Manifest> = None;

    // Go through the directories and install the mods
    for dir in directories {
        let mut new_dir_name = "".to_string();

        // Find the manifest.json file in the directory and add the mod to our mods file
        for entry in fs::read_dir(&dir).unwrap() {
            match entry {
                Ok(entry) => {
                    if entry
                        .file_name()
                        .to_string_lossy()
                        .contains("manifest.json")
                    {
                        manifest = Some(get_manifest(&entry.path()));
                        new_dir_name = add_mod_through_manifest(
                            manifest.clone().unwrap().clone(),
                            group_name.clone(),
                        )
                        .await;
                    }
                }
                Err(_) => {
                    continue;
                }
            }
        }

        // Rename the directory to the mod name
        let mut renamed_dir = outpath.clone();
        if depth < &3 {
            renamed_dir.pop();
        }
        renamed_dir.push(&new_dir_name);
        let rename_result = fs::rename(&dir, &renamed_dir).ok();
        if rename_result.is_none() {
            return Err("Failed to rename directory".to_string());
        }

        let game_path = paths::mod_path().join(format!(".{}", &manifest.clone().unwrap().name));

        // If the mod directory in game path already exists, remove it
        if &game_path.exists() == &true {
            fs::remove_dir_all(&game_path).unwrap();
        }

        // Move the renamed directory to the game path
        let move_result = fs::rename(&renamed_dir, &game_path);

        match move_result {
            Ok(_) => {
                // If the temp directory still exists, remove it
                if renamed_dir.clone().exists() {
                    fs::remove_dir_all(&renamed_dir).unwrap();
                }
            }
            Err(_) => {
                return Err("Failed to move directory".to_string());
            }
        }
    }

    Ok(())
}

/// Lists all subdirectories in unpacked dir
///
/// * `path` - The path to the directory
///
/// # Returns a vector with all subdirectories
fn list_dirs_in_directory(path: &Path) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = Vec::new();
    if path.is_dir() {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry;
            if entry.is_err() {
                continue;
            }
            let path = entry.unwrap().path();
            if path.is_dir() {
                paths.push(path);
            }
        }
    }
    paths
}

/// Calculates the percentage of the current installation progress
///
/// * `value` - The current value
/// * `max_value` - The maximum value
///
/// # Returns the percentage of the current installation progress
fn calculate_percentage(value: &usize, max_value: &usize) -> usize {
    if max_value == &0 {
        return 0;
    }
    let scaled_value = value * 10;
    let result = scaled_value / max_value;
    let remainder = scaled_value % max_value;
    if remainder >= max_value / 2 {
        return result + 1;
    }
    result
}

/// Creates the installation progress bar
///
/// * `amount` - The current amount of installed files
/// * `max` - The maximum amount of files
pub fn install_progress(amount: &usize, max: &usize) -> String {
    let mut new_console_output =
        "<span class=\"console-gray\">[Junimo] Install progress [".to_string();
    let calculated_percentage = calculate_percentage(amount, max);
    for rin in 0..calculated_percentage {
        new_console_output = format!("{}█", new_console_output);
    }
    for rin in calculated_percentage..10 {
        new_console_output = format!("{}⠀", new_console_output);
    }
    format!("{}]</span>", new_console_output)
}

/// Gets the mods manifest file
///
/// * `path` - The path to the manifest file
///
/// # Returns the mods manifest file
pub fn get_manifest(path: &PathBuf) -> Manifest {
    let mut file = File::open(path).unwrap();
    let mut output = String::new();
    file.read_to_string(&mut output).unwrap();
    output = output.replace("UniqueId", "UniqueID");
    output = output.replace("Authour", "Author");
    json_strip_comments::strip(&mut output).unwrap();
    match extract_json(&output) {
        Some(json) => output = json,
        None => println!("No JSON found"),
    }
    serde_json::from_str(&output).unwrap()
}

/// Extracts the JSON from the manifest file
///
/// * `input` - The input string
///
/// # Returns the JSON string
fn extract_json(input: &str) -> Option<String> {
    let start_pos = input.find('{');
    let end_pos = input.rfind('}');

    match (start_pos, end_pos) {
        (Some(start), Some(end)) if end > start => Some(input[start..=end].to_string()),
        _ => None,
    }
}

/// Adds a mod through the manifest file
///
/// * `manifest` - The mods manifest file
/// * `group_name` - The name of the group. None if it's not a group
///
/// # Returns the name of the mod
async fn add_mod_through_manifest(manifest: Manifest, group_name: Option<String>) -> String {
    let mut dependencies: Vec<Dependency> = Vec::new();

    // Select the dependencies from the manifest file
    match manifest.dependencies {
        Some(deps) => {
            for dep in deps {
                dependencies.push(dep);
            }
        }
        None => {}
    }

    // Select the content pack from the manifest file
    match manifest.content_pack {
        Some(content_pack) => {
            dependencies.push(content_pack);
        }
        None => {}
    }

    // Create mod info for the mod
    let mut new_mod = ModInfo {
        name: manifest.name,
        summary: manifest.description.clone(),
        description: manifest.description,
        picture_url: None,
        mod_downloads: 0,
        mod_unique_downloads: 0,
        uid: 0,
        mod_id: 0,
        game_id: 1303,
        allow_rating: false,
        domain_name: "stardewvalley".to_owned(),
        category_id: 0,
        version: manifest.version.to_detailed(),
        endorsement_count: 0,
        created_timestamp: 0,
        created_time: "".to_owned(),
        updated_timestamp: 0,
        updated_time: "".to_owned(),
        author: manifest.author.clone().unwrap(),
        uploaded_by: manifest.author.unwrap(),
        uploaded_users_profile_url: "".to_owned(),
        contains_adult_content: false,
        status: "".to_owned(),
        available: true,
        unique_id: Some(manifest.unique_id),
        more_info: None,
        dependencies: Some(dependencies),
        group: group_name,
        is_broken: None,
    };

    // Check for compatibilities and update the mod info
    let config = config::get_config(paths::config_path());
    if config.activate_broken.is_none() || config.activate_broken.unwrap() {
        match compatibility::get_compability(vec![new_mod.clone()]).await {
            Some(compatibility) => {
                new_mod = compatibility[0].clone();
            }
            None => {}
        }
    }

    // Insert the mod info into the mods file
    insert_mod_info(&new_mod);
    new_mod.name
}

/// Inserts the mod info into the mods file. If the mod already exists, update it
///
/// * `infos` - The mod info
pub fn insert_mod_info(infos: &ModInfo) {
    let mut mod_list = get_all_mods();

    if mod_list
        .iter()
        .any(|mod_info| mod_info.name == infos.name && mod_info.version == infos.version)
    {
        return;
    } else if mod_list
        .iter()
        .any(|mod_info| mod_info.name == infos.name && mod_info.version != infos.version)
    {
        let index = mod_list
            .iter()
            .position(|mod_info| mod_info.name == infos.name)
            .unwrap();
        mod_list[index] = infos.clone();
        save_mods(mod_list);
    } else {
        mod_list.push(infos.clone());
        save_mods(mod_list);
    }
}
