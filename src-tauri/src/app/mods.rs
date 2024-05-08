use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use rfd::FileDialog;
use zip::read::ZipArchive;
use std::{fs, thread};
use regex::Regex;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, command, Manager, api};
use url::Url;
use crate::app::{console, mod_installation, mods, profiles};
use crate::app::utility::{browser, paths, zips};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModInfo {
    pub name: String,
    summary: String,
    description: String,
    picture_url: Option<String>,
    mod_downloads: u64,
    mod_unique_downloads: u64,
    uid: u64,
    mod_id: u32,
    game_id: u32,
    allow_rating: bool,
    domain_name: String,
    category_id: u32,
    pub version: String,
    endorsement_count: u32,
    created_timestamp: u64,
    created_time: String,
    updated_timestamp: u64,
    updated_time: String,
    author: String,
    uploaded_by: String,
    uploaded_users_profile_url: String,
    contains_adult_content: bool,
    status: String,
    available: bool,
    pub unique_id: Option<String>,
    more_info: Option<String>,
    dependencies: Option<Vec<Dependency>>,
}

impl PartialEq for ModInfo {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Manifest {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Author")]
    author: String,
    #[serde(rename = "Version")]
    version: String,
    #[serde(rename = "Description")]
    description: String,
    #[serde(rename = "UniqueID")]
    pub unique_id: String,
    #[serde(rename = "EntryDll")]
    entry_dll: Option<String>,
    #[serde(rename = "MinimumApiVersion")]
    minimum_api_version: Option<String>,
    #[serde(rename = "UpdateKeys")]
    update_keys: Option<Vec<String>>,
    #[serde(rename = "ContentPackFor")]
    pub content_pack: Option<Dependency>,
    #[serde(rename = "Dependencies")]
    pub dependencies: Option<Vec<Dependency>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Dependency {
    #[serde(rename = "UniqueID")]
    pub unique_id: String,
    #[serde(rename = "MinimumVersion")]
    minimum_version: Option<String>,
    #[serde(rename = "IsRequired")]
    is_required: Option<bool>,
}

pub fn save_mods(mod_infos: Vec<ModInfo>) {
    let json = serde_json::to_string(&mod_infos).unwrap();
    let mut file = File::create(paths::mod_json_path()).expect("Fail");
    file.write_all(json.as_bytes()).unwrap();
}

#[command]
pub async fn open_search_browser(handle: AppHandle) {
    let nexus_link = "https://www.nexusmods.com/stardewvalley";
    browser::open_url(nexus_link);
}

#[command]
pub fn get_installed_mods() -> Vec<ModInfo> {
    let path = paths::mod_json_path();
    let current_profile = profiles::get_current_profile();

    if !path.exists() {
        let mods: Vec<ModInfo> = Vec::new();
        save_mods(mods);
    }

    let all_mods = get_all_mods();
    all_mods.iter().filter(|mod_info| !current_profile.mods.contains(mod_info)).cloned().collect()
}

pub fn get_all_mods() -> Vec<ModInfo> {
    let path = paths::mod_json_path();

    if !path.exists() {
        let mods: Vec<ModInfo> = Vec::new();
        save_mods(mods);
    }

    let data_raw = tauri::api::file::read_string(path).unwrap();
    let data = data_raw.as_str();
    serde_json::from_str(data).unwrap()
}

pub fn insert_mod_info(infos: &ModInfo) {
    let mut mod_list = get_all_mods();

    if mod_list.iter().any(|mod_info| mod_info.name == infos.name && mod_info.version == infos.version) {
        return;
    }
    else if mod_list.iter().any(|mod_info| mod_info.name == infos.name && mod_info.version != infos.version) {
        let index = mod_list.iter().position(|mod_info| mod_info.name == infos.name).unwrap();
        mod_list[index] = infos.clone();
        save_mods(mod_list);
    }
    else {
        mod_list.push(infos.clone());
        save_mods(mod_list);
    }
}

#[command]
pub fn add_mod(app_handle: AppHandle) {
    let file = FileDialog::new()
        .add_filter("zip", &["zip"])
        .add_filter("rar", &["rar"])
        .set_directory("/")
        .pick_file();

    if let Some(path) = file {
        mod_installation::start_installation(app_handle.clone(), &path);
    } else {
        println!("No file was selected.");
    }
}

#[command]
pub fn uninstall_mods(app_handle: AppHandle, mods: Vec<&str>) {
    for mod_name in mods {
        uninstall_mod(app_handle.clone(), mod_name);
    }
}

#[command]
pub fn uninstall_mod(app_handle: AppHandle, name: &str) {
    let mut mods = get_all_mods();
    mods.retain(|mod_info| mod_info.name != name);
    save_mods(mods);

    let mut profile = profiles::get_profiles();
    let mut new_profiles = Vec::new();
    for p in profile {
        if p.mods.iter().any(|mod_info| mod_info.name == name) {
            let mut new_mods = p.mods.clone();
            new_mods.retain(|mod_info| mod_info.name != name);
            let new_profile = profiles::Profile {
                name: p.name,
                mods: new_mods,
                currently: p.currently
            };
            new_profiles.push(new_profile);
        } else {
            new_profiles.push(p);
        }
    }
    profiles::save_profiles(&new_profiles);

    let mut mod_path = paths::mod_path();
    let path = format!("{}/{}.zip", mod_path.display(), name);
    if Path::new(&path).exists() {
        fs::remove_file(path).expect("Failed to remove file");
    }
}

pub fn unpack_manifest(zip_path: &PathBuf, name: &str) {
    let path = paths::mod_path();
    let zip = File::open(zip_path).unwrap();
    let manifest_archive = ZipArchive::new(zip).unwrap();

    zips::extract_manifest(manifest_archive, &path, name).unwrap();
    insert_unique(name);
}

pub fn get_manifest(path: &PathBuf) -> Manifest {
    let mut file = File::open(path).unwrap();
    let mut output = String::new();
    file.read_to_string(&mut output).unwrap();
    output = output.replace("UniqueId", "UniqueID");
    json_strip_comments::strip(&mut output).unwrap();
    match extract_json(&output) {
        Some(json) =>  output = json,
        None => println!("No JSON found"),
    }
    println!("{}", output);
    serde_json::from_str(&output).unwrap()
}

pub fn get_dependencies(path: &PathBuf) -> Option<Vec<Dependency>> {
    let mut file = File::open(path).unwrap();
    let mut output = String::new();
    file.read_to_string(&mut output).unwrap();
    output = output.replace("UniqueId", "UniqueID");
    json_strip_comments::strip(&mut output).unwrap();
    match extract_json(&output) {
        Some(json) =>  {
            output = json;
            let manifest: Manifest = serde_json::from_str(&output).unwrap();
            manifest.dependencies
        },
        None => None
    }
}

pub fn add_mod_through_manifest(path: &PathBuf) -> String {
    let mut file = File::open(path).unwrap();
    let mut output = String::new();
    file.read_to_string(&mut output).unwrap();
    output = output.replace("UniqueId", "UniqueID");
    json_strip_comments::strip(&mut output).unwrap();
    match extract_json(&output) {
        Some(json) =>  output = json,
        None => println!("No JSON found"),
    }
    let manifest: Manifest = serde_json::from_str(&output).unwrap();

    let mut dependencies: Vec<Dependency> = Vec::new();

    match manifest.dependencies {
        Some(deps) => {
            for dep in deps {
                dependencies.push(dep);
            }
        },
        None => {
            println!("No dependencies found");
        }
    }

    match manifest.content_pack {
        Some(content_pack) => {
            dependencies.push(content_pack);
        },
        None => {
            println!("No content pack found");
        }
    }

    let new_mod = ModInfo {
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
        version: manifest.version,
        endorsement_count: 0,
        created_timestamp: 0,
        created_time: "".to_owned(),
        updated_timestamp: 0,
        updated_time: "".to_owned(),
        author: manifest.author.clone(),
        uploaded_by: manifest.author,
        uploaded_users_profile_url: "".to_owned(),
        contains_adult_content: false,
        status: "".to_owned(),
        available: true,
        unique_id: Some(manifest.unique_id),
        more_info: None,
        dependencies: Some(dependencies)
    };

    insert_mod_info(&new_mod);
    new_mod.name
}

fn extract_json(input: &str) -> Option<String> {
    let start_pos = input.find('{');
    let end_pos = input.rfind('}');

    match (start_pos, end_pos) {
        (Some(start), Some(end)) if end > start => {
            Some(input[start..=end].to_string())
        },
        _ => None, // Return None if no valid JSON is found
    }
}

pub fn insert_unique(name: &str) {
    let mods = get_all_mods();
    let path = paths::mod_path();

    let manifest_path = format!("{}/{}/manifest.json", path.display(), name);
    let mut file = File::open(manifest_path).unwrap();
    let mut output = String::new();
    file.read_to_string(&mut output).unwrap();
    output = output.replace("UniqueId", "UniqueID");
    json_strip_comments::strip(&mut output).unwrap();
    match extract_json(&output) {
        Some(json) =>  output = json,
        None => println!("No JSON found"),
    }
    let manifest: Manifest = serde_json::from_str(&output).unwrap();

    let mut new_mods = Vec::new();
    for mod_info in mods {
        if mod_info.name == name {
            let mut new_mod_info = mod_info.clone();
            new_mod_info.unique_id = Some(manifest.unique_id.clone());
            new_mod_info.dependencies = manifest.dependencies.clone();
            new_mods.push(new_mod_info);
        }
        else {
            new_mods.push(mod_info);
        }
    }

    save_mods(new_mods);
}

pub fn check_dependencies(mods: Vec<ModInfo>) -> Vec<ModInfo> {
    let cloned_mods = mods.clone();
    let mut new_modinfo = Vec::new();

    for mut mod_info in mods.clone() {
        if (mod_info.dependencies.is_none()) {
            new_modinfo.push(mod_info.clone());
            continue;
        }

        match &mod_info.dependencies {
            Some(dependencies) => {
                let mut mode = 0;
                for dependency in dependencies {
                    if cloned_mods.iter().any(|mod_info| mod_info.unique_id == Some(dependency.unique_id.clone())) {
                        mod_info.more_info = None;
                        continue;
                    }
                    if dependency.is_required == None || dependency.is_required == Some(true) {
                        let split = dependency.unique_id.split('.').collect::<Vec<&str>>();
                        mod_info.more_info = Some(format!("<span style=\"color: #cf3838\">Missing mod: {}</span>", split[split.len() - 1]));
                        break;
                    }
                    else {
                        if mode == 2 {
                            continue;
                        }

                        let split = dependency.unique_id.split('.').collect::<Vec<&str>>();
                        mod_info.more_info = Some(format!("<span style=\"color: #f5d442\">Recommended mod: {}</span>", split[split.len() - 1]));
                        continue;
                    }
                }
                new_modinfo.push(mod_info.clone());
            }
            None => {
                println!("Hier 3");
            }
        }
    }

    new_modinfo
}

pub fn any_missing_dependencies(mods: &Vec<ModInfo>) -> bool {
    let cloned_mods = mods.clone();
    let mut new_modinfo = Vec::new();

    for mut mod_info in mods.clone() {
        if (mod_info.dependencies.is_none()) {
            new_modinfo.push(mod_info.clone());
            continue;
        }

        match &mod_info.dependencies {
            Some(dependencies) => {
                for dependency in dependencies {
                    if cloned_mods.iter().any(|mod_info| mod_info.unique_id == Some(dependency.unique_id.clone())) {
                        continue;
                    }
                    if dependency.is_required == None || dependency.is_required == Some(true) {
                        return true;
                    }
                    else {
                        continue;
                    }
                }
                new_modinfo.push(mod_info.clone());
            }
            None => {
                println!("Hier 3");
            }
        }
    }
    false
}