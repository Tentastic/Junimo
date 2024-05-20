use std::collections::{HashMap, HashSet};
use crate::app::utility::{browser, paths, zips};
use crate::app::{config, console, mod_installation, mods, profiles};
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tauri::{command, AppHandle, Manager};
use walkdir::WalkDir;
use zip::read::ZipArchive;
use crate::app::api::compatibility;
use crate::app::profiles::Profile;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModInfo {
    pub name: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub picture_url: Option<String>,
    pub mod_downloads: u64,
    pub mod_unique_downloads: u64,
    pub uid: u64,
    pub mod_id: u32,
    pub game_id: u32,
    pub allow_rating: bool,
    pub domain_name: String,
    pub category_id: u32,
    pub version: String,
    pub endorsement_count: u32,
    pub created_timestamp: u64,
    pub created_time: String,
    pub updated_timestamp: u64,
    pub updated_time: String,
    pub author: String,
    pub uploaded_by: String,
    pub uploaded_users_profile_url: String,
    pub contains_adult_content: bool,
    pub status: String,
    pub available: bool,
    pub unique_id: Option<String>,
    pub more_info: Option<String>,
    pub dependencies: Option<Vec<Dependency>>,
    pub group: Option<String>,
    pub is_broken: Option<bool>
}

impl PartialEq for ModInfo {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Hash for ModInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state); // Only hash the name
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Manifest {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Author")]
    pub author: Option<String>,
    #[serde(rename = "Version")]
    pub version: Version,
    #[serde(rename = "Description")]
    pub description: Option<String>,
    #[serde(rename = "UniqueID")]
    pub unique_id: String,
    #[serde(rename = "EntryDll")]
    pub entry_dll: Option<String>,
    #[serde(rename = "MinimumApiVersion")]
    pub minimum_api_version: Option<String>,
    #[serde(rename = "UpdateKeys")]
    pub update_keys: Option<Vec<String>>,
    #[serde(rename = "ContentPackFor")]
    pub content_pack: Option<Dependency>,
    #[serde(rename = "Dependencies")]
    pub dependencies: Option<Vec<Dependency>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Version {
    Simple(String),
    Detailed {
        #[serde(rename = "MajorVersion")]
        major: i32,
        #[serde(rename = "MinorVersion")]
        minor: i32,
        #[serde(rename = "PatchVersion")]
        patch_version: i32,
        #[serde(rename = "Build")]
        build: Option<String>,
    },
}

impl Version {
    pub fn to_detailed(&self) -> String {
        match self {
            Version::Simple(s) => s.clone(),
            Version::Detailed { major, minor, patch_version, build } => {
                format!("{}.{}.{}", major, minor, patch_version)
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Dependency {
    #[serde(rename = "UniqueID")]
    pub unique_id: String,
    #[serde(rename = "MinimumVersion")]
    pub minimum_version: Option<String>,
    #[serde(rename = "IsRequired")]
    pub is_required: Option<bool>,
}

/// Saves the mods to the mods.json file in the appdata folder
pub fn save_mods(mod_infos: Vec<ModInfo>) {
    let json = serde_json::to_string(&mod_infos).unwrap();
    let mut file = File::create(paths::mod_json_path()).expect("Fail");
    file.write_all(json.as_bytes()).unwrap();
}

/// Tauri command to open the Stardew Valley Nexus page in the default browser
#[command]
pub async fn open_search_browser(handle: AppHandle) {
    let nexus_link = "https://www.nexusmods.com/stardewvalley";
    browser::open_url(nexus_link);
}

/// Tauri command to get all installed mods that are not used in the current profile
#[command]
pub async fn get_installed_mods(app: tauri::AppHandle) -> Vec<ModInfo> {
    let path = paths::mod_json_path();
    let current_profile = profiles::get_current_profile(paths::profile_path()).await;

    if !path.exists() {
        let mods: Vec<ModInfo> = Vec::new();
        save_mods(mods);
    }

    let mut all_mods = get_all_mods();
    all_mods = all_mods
        .iter()
        .filter(|mod_info| !current_profile.mods.contains(mod_info))
        .cloned()
        .collect();
    all_mods.sort_by(|a, b| a.name.cmp(&b.name));
    all_mods
}

/// Gets all mods from the mods.json file in the appdata folder
pub fn get_all_mods() -> Vec<ModInfo> {
    let path = paths::mod_json_path();

    if !path.exists() {
        let mods: Vec<ModInfo> = Vec::new();
        save_mods(mods);
    }

    let data_raw = fs::read_to_string(path).unwrap();
    let data = data_raw.as_str();
    serde_json::from_str(data).unwrap()
}

/// Adds mod
#[command]
pub async fn add_mod(app_handle: AppHandle) {
    // Opens File Dialog to select a mod's zip file
    let file = FileDialog::new()
        .add_filter("zip", &["zip"])
        .set_directory("/")
        .pick_file();

    // Starts the installation of the zip file.
    if let Some(path) = file {
        mod_installation::start_installation(app_handle.clone(), &path).await;
    } else {
        println!("No file was selected.");
    }
}

/// Uninstalls multiple mods
#[command]
pub fn uninstall_mods(app_handle: AppHandle, mods: Vec<&str>) {
    for mod_name in mods {
        uninstall_mod(app_handle.clone(), mod_name);
    }
}

/// Uninstalls a mod by removing it from the mods.json file, all profiles and deleting the mod folder
#[command]
pub fn uninstall_mod(app_handle: AppHandle, name: &str) {
    // Remove mod from mods.json
    let mut mods = get_all_mods();
    mods.retain(|mod_info| mod_info.name != name);
    save_mods(mods);

    // Remove mod from all profiles
    let mut profile = profiles::get_profiles(paths::profile_path());
    let mut new_profiles = Vec::new();
    for p in profile {
        if p.mods.iter().any(|mod_info| mod_info.name == name) {
            let mut new_mods = p.mods.clone();
            new_mods.retain(|mod_info| mod_info.name != name);
            let new_profile = profiles::Profile {
                name: p.name,
                mods: new_mods,
                currently: p.currently,
            };
            new_profiles.push(new_profile);
        } else {
            new_profiles.push(p);
        }
    }
    profiles::save_profiles(&new_profiles, &paths::profile_path());

    // Remove mod from mods folder. If the mod is currently not used, the folder name is prefixed with a dot
    let mut mod_path = paths::mod_path();
    mod_path = mod_path.join(&name);
    remove_mod_folder(&app_handle, &mod_path, name);
    mod_path.pop();
    mod_path = mod_path.join(format!(".{}", name));
    remove_mod_folder(&app_handle, &mod_path, name);

    // Add a success line to the console
    console::add_line(
        &app_handle,
        format!("<span class=\"console-green\">[Junimo] Uninstalled: {}</span>", &name),
    );
}

/// Removes a mod folder
fn remove_mod_folder(app_handle: &AppHandle, mod_path: &PathBuf, name: &str) {
    if Path::new(&mod_path).exists() {
        let remove_result = fs::remove_dir_all(&mod_path);
        match remove_result {
            Ok(_) => {
            }
            Err(e) => {
                console::add_line(
                    &app_handle,
                    format!("<span class=\"console-red\">[Junimo] Failed to remove mod's folder: {}</span>", &name),
                );
                println!("{}", e);
                return;
            }
        }
    }
}

/// Check for compatibility errors in mods
pub async fn compatibility_check(app_handle: AppHandle) -> bool {
    let config = config::get_config(paths::config_path());
    let mut mods_result: Option<Vec<ModInfo>> = Some(get_all_mods());

    if config.activate_broken.is_none() || config.activate_broken.unwrap() {
        mods_result = compatibility::get_compability(get_all_mods()).await;
    }
    else {
    let mut mods = mods_result.clone().unwrap();
        for mod_info in mods.iter_mut() {
            mod_info.is_broken = None;
            mod_info.more_info = None;
        }
        save_mods(mods);
    }

    if config.activate_requirements.is_none() || config.activate_requirements.unwrap() {
        match mods_result {
            Some(mods) => {
                let profiles = profiles::get_profiles(paths::profile_path());

                let hash_map: HashMap<_, _> = mods.clone().into_iter().enumerate()
                    .map(|(index, modinfo)| (modinfo.unique_id, index))
                    .collect();

                let mut new_profiles: Vec<Profile> = vec![];
                for profile in profiles {
                    let mut new_profile: Profile = profile.clone();
                    let mut new_profile_mods: Vec<ModInfo> = vec![];
                    for mod_info in profile.mods {
                        if hash_map.contains_key(&mod_info.unique_id) {
                            let index = hash_map.get(&mod_info.unique_id).unwrap();
                            let new_mod: ModInfo = mods[*index].clone();
                            new_profile_mods.push(new_mod);
                        }
                        else {
                            new_profile_mods.push(mod_info);
                        }
                    }
                    new_profile.mods = new_profile_mods;
                    new_profiles.push(new_profile);
                }
                profiles::save_profiles(&new_profiles, &paths::profile_path());

                save_mods(mods);
                true
            }
            None => {
                false
            }
        }
    }
    else {
        true
    }
}

pub fn check_dependencies(mods: Vec<ModInfo>) -> Vec<ModInfo> {
    let cloned_mods = mods.clone();
    let mut new_modinfo = Vec::new();

    // Create hashset of all unique_ids
    let unique_ids: HashSet<Option<String>> = mods.iter().map(|mod_info| mod_info.unique_id.clone()).collect();

    for mut mod_info in mods.clone() {
        if mod_info.dependencies.is_none() || mod_info.more_info.is_some() {
            if !(mod_info.more_info.is_some() &&
                (mod_info.more_info.clone().unwrap().contains("Missing") || mod_info.more_info.clone().unwrap().contains("Recommended"))) {
                new_modinfo.push(mod_info.clone());
                continue;
            }
        }

        match &mod_info.dependencies {
            Some(dependencies) => {
                let mut mode = 0;
                for dependency in dependencies {
                    if unique_ids.contains(&Some(dependency.unique_id.clone())) {
                        mod_info.more_info = None;
                        continue;
                    }
                    if dependency.is_required == None || dependency.is_required == Some(true) {
                        let split = dependency.unique_id.split('.').collect::<Vec<&str>>();

                        mod_info.more_info = Some(format!(
                            "<span style=\"color: #cf3838\">Missing mod: {}</span>",
                            dependency.unique_id.replace(format!("{}.", split[0]).as_str(), "")
                        ));
                        break;
                    } else {
                        if mode == 2 {
                            continue;
                        }

                        let split = dependency.unique_id.split('.').collect::<Vec<&str>>();
                        mod_info.more_info = Some(format!(
                            "<span class=\"console-blue\">Recommended mod: {}</span>",
                            split[split.len() - 1]
                        ));
                        continue;
                    }
                }
                new_modinfo.push(mod_info.clone());
            }
            None => {
            }
        }
    }

    new_modinfo
}