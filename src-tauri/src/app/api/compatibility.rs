use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;
use serde::{Deserialize, Serialize};
use url::Url;
use widestring::U16CString;
use winapi::shared::minwindef::{DWORD, LPVOID};
use winapi::um::winnt::HANDLE;
use winapi::um::winver::{GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW};
use crate::app::{mods, profiles};
use crate::app::mods::ModInfo;
use crate::app::utility::paths;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SmapiMod {
    #[serde(rename = "id")]
    id: String,
    #[serde(rename = "installedVersion")]
    installed_version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SmapiPostWrapper {
    #[serde(rename = "mods")]
    mods: Vec<SmapiMod>,
    #[serde(rename = "apiVersion")]
    api_version: Option<String>,
    #[serde(rename = "gameVersion")]
    game_version: String,
    platform: String,
    #[serde(rename = "includeExtendedMetadata")]
    include_extend_metadata: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SmapiResult {
    mods: Vec<SmapiWrapper>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SmapiWrapper {
    id: String,
    #[serde(rename = "suggestedUpdate")]
    suggested_update: Option<SuggestesUpdate>,
    metadata: SmapiMeta,
    errors: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SuggestesUpdate {
    version: String,
    url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SmapiMeta {
    id: Vec<String>,
    name: Option<String>,
    #[serde(rename = "nexusID")]
    nexus_id: Option<i32>,
    #[serde(rename = "gitHubRepo")]
    github_repo: Option<String>,
    #[serde(rename = "customSourceUrl")]
    custom_source_url: Option<String>,
    main: Option<SmapiMain>,
    optional: Option<SmapiMain>,
    unofficial: Option<SmapiMain>,
    #[serde(rename = "unofficialForBeta")]
    unofficial_for_beta: Option<String>,
    #[serde(rename = "hasBetaInfo")]
    has_beta_info: Option<String>,
    #[serde(rename = "compatibilityStatus")]
    compability_status: Option<String>,
    #[serde(rename = "compatibilitySummary")]
    compability_summary: Option<String>,
    #[serde(rename = "brokeIn")]
    broke_in: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SmapiMain {
    version: String,
    url: String,
}

pub async fn get_compability(app: tauri::AppHandle, mods: Vec<ModInfo>) -> Option<Vec<ModInfo>> {
    let url = Url::parse("https://smapi.io/api/v3.0/mods").unwrap();

    let path = paths::get_game_path().join("StardewModdingAPI.dll").to_string_lossy().to_string();
    let game_path = paths::get_game_path().join("Stardew Valley.exe").to_string_lossy().to_string();
    let dll_path = path.as_str();

    let mut api_version: Option<String> = None;
    match get_version_info_from_dll(dll_path) {
        Some(version) => api_version = Some(format!("{}.{}.{}", version.major, version.minor, version.patch)),
        None => api_version = None,
    }

    let mut game_version = "".to_string();
    match get_version_info_from_dll(game_path.as_str()) {
        Some(version) => game_version  = format!("{}.{}.{}", version.major, version.minor, version.patch),
        None => println!("Failed to get version information"),
    }

    let mut mods_post: Vec<SmapiMod> = vec![];

    for mod_info in mods.clone() {
        match mod_info.unique_id {
            Some(unique) => {
                let smapi_mod = SmapiMod {
                    id: unique.clone(),
                    installed_version: mod_info.version,
                };
                println!("Some: {}", unique.clone());
                mods_post.push(smapi_mod);
            }
            None => {
                println!("None: {}", mod_info.name);
            }
        }
    }

    let post_body_wrapper = SmapiPostWrapper {
        mods: mods_post,
        api_version,
        game_version,
        #[cfg(target_os = "windows")]
        platform: "Windows".to_string(),
        #[cfg(target_os = "linux")]
        platform: "Linux".to_string(),
        #[cfg(target_os = "macos")]
        platform: "Mac".to_string(),
        include_extend_metadata: true,
    };

    let body = serde_json::to_string(&post_body_wrapper).unwrap();

    let client = reqwest::Client::new();
    let res = client
        .post(url)
        .body(body)
        .header("Application-Name", "application/json")
        .header("Application-Version", app.package_info().version.clone().to_string())
        .header("User-Agent", "Junimo")
        .header("Content-Type", "application/json")
        .header("accept", "application/json")
        .send()
        .await
        .unwrap();

    if res.status().is_success() {
        match res.text().await {
            Ok(text) => {
                println!("{}", text.clone().as_str());
                let result: Result<Vec<SmapiWrapper>, serde_json::Error> = serde_json::from_str(text.as_str());
                match result {
                    Ok(wrapper) => {
                        let mut new_mods: Vec<ModInfo> = mods.clone();

                        let hash_map: HashMap<_, _> = mods.clone().into_iter().enumerate()
                            .map(|(index, modinfo)| (modinfo.unique_id, index))
                            .collect();

                        for mod_info in wrapper {
                            if hash_map.contains_key(&Some(mod_info.id.clone())) {
                                let get_index = hash_map.get(&Some(mod_info.id.clone())).unwrap();
                                let mut copied_mod = mods[*get_index].clone();

                                if mods[*get_index].more_info.is_none() {
                                    match mod_info.metadata.compability_status {
                                        Some(status) => {
                                            if status != "Ok" {
                                                copied_mod.more_info = mod_info.metadata.compability_summary;
                                                new_mods[*get_index] = copied_mod.clone();
                                            }
                                        }
                                        None => {
                                            new_mods[*get_index] = copied_mod.clone();
                                        }
                                    }
                                }
                                new_mods[*get_index] = copied_mod;
                            }
                        }

                        return Some(new_mods);
                    },
                    Err(e) => return None,
                }
            }
            Err(e) => {
                return None
            }
        }
    }
    else {
        return None
    }
}

#[derive(Debug)]
struct Version {
    major: u16,
    minor: u16,
    patch: u16,
    build: u16,
}

#[repr(C)]
struct VS_FIXEDFILEINFO {
    dwSignature: DWORD,
    dwStrucVersion: DWORD,
    dwFileVersionMS: DWORD,
    dwFileVersionLS: DWORD,
    dwProductVersionMS: DWORD,
    dwProductVersionLS: DWORD,
    dwFileFlagsMask: DWORD,
    dwFileFlags: DWORD,
    dwFileOS: DWORD,
    dwFileType: DWORD,
    dwFileSubtype: DWORD,
    dwFileDateMS: DWORD,
    dwFileDateLS: DWORD,
}

fn get_version_info_from_dll(path: &str) -> Option<Version> {
    // Convert path to a wide string
    let wide_path: Vec<u16> = OsString::from(path).encode_wide().chain(std::iter::once(0)).collect();

    // Get the size of the version info
    let mut handle: DWORD = 0;
    let size = unsafe { GetFileVersionInfoSizeW(wide_path.as_ptr(), &mut handle) };
    if size == 0 {
        return None;
    }

    // Read the version info into a buffer
    let mut buffer: Vec<u8> = vec![0; size as usize];
    let result = unsafe { GetFileVersionInfoW(wide_path.as_ptr(), handle as usize as DWORD, size, buffer.as_mut_ptr() as *mut _) };
    if result == 0 {
        return None;
    }

    // Extract the version information
    let mut lp_buffer: LPVOID = std::ptr::null_mut();
    let mut len: u32 = 0;
    let sub_block = U16CString::from_str("\\").unwrap();
    let result = unsafe { VerQueryValueW(buffer.as_ptr() as *const _, sub_block.as_ptr(), &mut lp_buffer, &mut len) };
    if result == 0 {
        return None;
    }

    // Safely dereference the pointer without moving the content
    unsafe {
        let version_info: &VS_FIXEDFILEINFO = &*(lp_buffer as *const VS_FIXEDFILEINFO);
        if version_info.dwSignature != 0xfeef04bd {
            return None;
        }

        Some(Version {
            major: (version_info.dwFileVersionMS >> 16) as u16,
            minor: (version_info.dwFileVersionMS & 0xFFFF) as u16,
            patch: (version_info.dwFileVersionLS >> 16) as u16,
            build: (version_info.dwFileVersionLS & 0xFFFF) as u16,
        })
    }
}