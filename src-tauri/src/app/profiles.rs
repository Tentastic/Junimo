use crate::app::models::mod_info::ModInfo;
use crate::app::utility::paths;
use crate::app::{config, mods};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tauri::{command, Manager, Runtime, WebviewUrl};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Profile {
    pub name: String,
    pub mods: Vec<ModInfo>,
    pub currently: bool,
}

#[command]
pub async fn open_profile<R: Runtime>(handle: tauri::AppHandle<R>) {
    #[cfg(target_os = "windows")]
    tauri::WebviewWindowBuilder::new(&handle, "Profiles", WebviewUrl::App("/profiles".into()))
        .title("Profiles")
        .transparent(true)
        .build()
        .unwrap();

    #[cfg(target_os = "unix")]
    tauri::WebviewWindowBuilder::new(&handle, "Profiles", WebviewUrl::App("/profiles".into()))
        .title("Profiles")
        .build()
        .unwrap();
}

#[command]
pub fn get_profiles(path: PathBuf) -> Vec<Profile> {
    check_path(&path);

    let data_raw = fs::read_to_string(path).unwrap();
    let data = data_raw.as_str();
    serde_json::from_str(data).unwrap()
}

pub fn save_profiles(profiles: &Vec<Profile>, path: &PathBuf) {
    let json = serde_json::to_string(&profiles).unwrap();
    let mut file = File::create(path).expect("Fail");
    file.write_all(json.as_bytes()).unwrap();
}

#[command]
pub async fn get_current_profile(path: PathBuf) -> Profile {
    check_path(&path);

    let data_raw = fs::read_to_string(path).unwrap();
    let data = data_raw.as_str();
    let profiles: Vec<Profile> = serde_json::from_str(data).unwrap();

    let config = config::get_config(paths::config_path());

    let mut return_profile: Profile = profiles[0].clone();
    for profile in profiles {
        if profile.currently {
            let mut new_profile = profile;
            if config.activate_requirements.is_none() || config.activate_requirements.unwrap() {
                new_profile.mods = mods::check_dependencies(new_profile.clone().mods);
            }

            /*match compatibility::get_compability(app.clone(), new_profile.clone().mods).await {
                Some(compability) => {
                    new_profile.mods = compability;
                }
                None => {}
            }*/

            return_profile = new_profile.clone();
        }
    }
    return_profile
        .mods
        .sort_by(|info1, info2| info1.name.cmp(&info2.name));

    return_profile
}

pub fn check_path(path: &PathBuf) -> bool {
    if !path.exists() {
        let mut profiles: Vec<Profile> = Vec::new();
        let profile = Profile {
            name: "Default".to_string(),
            mods: Vec::new(),
            currently: true,
        };
        profiles.push(profile);
        save_profiles(&profiles, &paths::profile_path());
        return false;
    }
    true
}

#[command]
pub fn change_current_profile<R: Runtime>(
    handle: tauri::AppHandle<R>,
    name: &str,
    path: PathBuf,
) -> Vec<Profile> {
    let profiles = get_profiles(path.clone());

    let mut new_profiles: Vec<Profile> = Vec::new();
    for profile in profiles {
        if profile.name == name {
            let new_profile = Profile {
                name: profile.name,
                mods: profile.mods,
                currently: true,
            };
            new_profiles.push(new_profile);
        } else {
            let new_profile = Profile {
                name: profile.name,
                mods: profile.mods,
                currently: false,
            };
            new_profiles.push(new_profile);
        }
    }
    handle
        .emit("profile-update", &new_profiles)
        .expect("Failed to emit event");
    save_profiles(&new_profiles, &path);
    new_profiles
}

#[command]
pub fn add_profile<R: Runtime>(
    handle: tauri::AppHandle<R>,
    name: &str,
    path: PathBuf,
) -> Vec<Profile> {
    let profiles = get_profiles(path.clone());

    let mut new_profiles: Vec<Profile> = Vec::new();
    for profile in profiles {
        let new_profile = Profile {
            name: profile.name,
            mods: profile.mods,
            currently: false,
        };
        new_profiles.push(new_profile);
    }
    let new_profile = Profile {
        name: name.to_string(),
        mods: Vec::new(),
        currently: true,
    };
    new_profiles.push(new_profile);
    handle
        .emit("profile-update", &new_profiles)
        .expect("Failed to emit event");
    save_profiles(&new_profiles, &path);
    new_profiles
}

#[command]
pub fn remove_profile<R: Runtime>(
    handle: tauri::AppHandle<R>,
    name: &str,
    path: PathBuf,
) -> Vec<Profile> {
    let profiles = get_profiles(path.clone());

    let mut new_profiles: Vec<Profile> = Vec::new();
    for profile in profiles {
        if profile.name != name {
            new_profiles.push(profile);
        }
    }
    handle
        .emit("profile-update", &new_profiles)
        .expect("Failed to emit event");
    save_profiles(&new_profiles, &path);
    new_profiles
}

#[command]
pub fn duplicate_profile(handle: tauri::AppHandle, from: String, name: String) -> Vec<Profile> {
    let profiles = get_profiles(paths::profile_path());

    let mut new_profiles: Vec<Profile> = Vec::new();
    let mut duplicate_profile: Profile = Profile {
        name,
        mods: vec![],
        currently: false,
    };
    for profile in profiles {
        if (profile.name == from) {
            duplicate_profile.mods = profile.clone().mods;
        }
        new_profiles.push(profile);
    }
    new_profiles.push(duplicate_profile);
    save_profiles(&new_profiles, &paths::profile_path());
    handle
        .emit("profile-update", &new_profiles)
        .expect("Failed to emit event");
    new_profiles
}

#[command]
pub fn modify_profile<R: Runtime>(
    handle: tauri::AppHandle<R>,
    name: &str,
    new_name: &str,
    path: PathBuf,
) -> Vec<Profile> {
    let profiles = get_profiles(path.clone());

    let mut new_profiles: Vec<Profile> = Vec::new();
    for profile in profiles {
        if profile.name == name {
            let new_profile = Profile {
                name: new_name.parse().unwrap(),
                mods: profile.mods,
                currently: profile.currently,
            };
            new_profiles.push(new_profile);
        } else {
            new_profiles.push(profile);
        }
    }
    handle
        .emit("profile-update", &new_profiles)
        .expect("Failed to emit event");
    save_profiles(&new_profiles, &path);
    new_profiles
}

#[command]
pub fn change_profile_mods(name: &str, mut mods: Vec<ModInfo>, path: PathBuf) {
    let profiles = get_profiles(path.clone());

    let mut seen = HashSet::new();
    mods.retain(|mod_info| seen.insert(mod_info.name.clone())); // Retains only if name is new to the set

    let mut new_profiles: Vec<Profile> = Vec::new();
    for profile in profiles {
        if profile.name == name {
            let new_profile = Profile {
                name: profile.name,
                mods: mods.clone(),
                currently: profile.currently,
            };
            new_profiles.push(new_profile);
        } else {
            new_profiles.push(profile);
        }
    }
    save_profiles(&new_profiles, &path);
}

#[cfg(test)]
mod tests {
    use crate::app::app_state::AppState;
    use tauri::test::mock_builder;
    use tempfile::tempdir;

    use super::*;

    fn create_app<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::App<R> {
        let (app_state, rx) = AppState::new();

        builder
            .invoke_handler(tauri::generate_handler![
                open_profile,
                get_current_profile,
                get_profiles,
                change_current_profile,
                add_profile,
                remove_profile,
                modify_profile,
                change_profile_mods
            ])
            .manage(app_state.clone())
            // remove the string argument to use your app's config file
            .build(tauri::generate_context!())
            .expect("failed to build app")
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct PathWrap {
        path: PathBuf,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct ModifyWrap {
        name: String,
        path: PathBuf,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct ModifyNameWrap {
        name: String,
        #[serde(rename = "newName")]
        new_name: String,
        path: PathBuf,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct ModWrap {
        name: String,
        path: PathBuf,
        mods: Vec<ModInfo>,
    }

    #[test]
    fn test_open_profile() {
        let app = create_app(mock_builder());
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();

        let res = tauri::test::get_ipc_response(
            &webview,
            tauri::webview::InvokeRequest {
                cmd: "open_profile".into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: "http://tauri.localhost".parse().unwrap(),
                body: tauri::ipc::InvokeBody::default(),
                headers: Default::default(),
            },
        );
        assert!(res.is_ok());
    }

    #[test]
    fn test_save_profile() {
        let tmp_dir = tempdir().unwrap();
        let profile_path = tmp_dir.path().join("profile.json");

        let mut profiles: Vec<Profile> = Vec::new();
        let profile = Profile {
            name: "Default".to_string(),
            mods: Vec::new(),
            currently: true,
        };
        profiles.push(profile);

        save_profiles(&profiles, &profile_path);
        assert!(profile_path.exists());
    }

    #[test]
    fn test_check_path() {
        let tmp_dir = tempdir().unwrap();
        let profile_path = tmp_dir.path().join("profile.json");
        let result = check_path(&profile_path);
        assert_eq!(result, false);
    }

    #[test]
    fn test_get_profiles() {
        let tmp_dir = tempdir().unwrap();
        let profile_path = tmp_dir.path().join("profile.json");

        let mut profiles: Vec<Profile> = Vec::new();
        let profile = Profile {
            name: "Default".to_string(),
            mods: Vec::new(),
            currently: true,
        };
        profiles.push(profile);

        save_profiles(&profiles, &profile_path);

        let app = create_app(mock_builder());
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();

        let path_wrap = PathWrap {
            path: profile_path.clone(),
        };
        let serialized_payload = serde_json::to_string(&path_wrap).unwrap();
        let invoke_body = tauri::ipc::InvokeBody::Json(serialized_payload.parse().unwrap());

        let res = tauri::test::get_ipc_response(
            &webview,
            tauri::webview::InvokeRequest {
                cmd: "get_profiles".into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: "http://tauri.localhost".parse().unwrap(),
                body: invoke_body,
                headers: Default::default(),
            },
        );

        assert!(res.is_ok());
        assert_eq!(
            res.unwrap()
                .deserialize::<Vec<Profile>>()
                .unwrap()
                .iter()
                .count(),
            profiles.iter().count()
        );
    }

    #[test]
    fn test_change_current_profile() {
        let tmp_dir = tempdir().unwrap();
        let profile_path = tmp_dir.path().join("profile.json");

        let mut profiles: Vec<Profile> = Vec::new();
        let profile = Profile {
            name: "Default".to_string(),
            mods: Vec::new(),
            currently: true,
        };
        let test_profile = Profile {
            name: "Test".to_string(),
            mods: Vec::new(),
            currently: false,
        };
        profiles.push(profile.clone());
        profiles.push(test_profile.clone());

        save_profiles(&profiles, &profile_path);

        let app = create_app(mock_builder());
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();

        let path_wrap = ModifyWrap {
            name: "Test".to_string(),
            path: profile_path.clone(),
        };
        let serialized_payload = serde_json::to_string(&path_wrap).unwrap();
        let invoke_body = tauri::ipc::InvokeBody::Json(serialized_payload.parse().unwrap());

        let res = tauri::test::get_ipc_response(
            &webview,
            tauri::webview::InvokeRequest {
                cmd: "change_current_profile".into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: "http://tauri.localhost".parse().unwrap(),
                body: invoke_body,
                headers: Default::default(),
            },
        );

        assert!(res.is_ok());
        assert_eq!(
            res.clone().unwrap().deserialize::<Vec<Profile>>().unwrap()[0].currently,
            false
        );
        assert_eq!(
            res.unwrap().deserialize::<Vec<Profile>>().unwrap()[1].currently,
            true
        );
    }

    #[test]
    fn test_add_profile() {
        let tmp_dir = tempdir().unwrap();
        let profile_path = tmp_dir.path().join("profile.json");

        let mut profiles: Vec<Profile> = Vec::new();
        let profile = Profile {
            name: "Default".to_string(),
            mods: Vec::new(),
            currently: true,
        };
        let test_profile = Profile {
            name: "Test".to_string(),
            mods: Vec::new(),
            currently: false,
        };
        profiles.push(profile.clone());

        save_profiles(&profiles, &profile_path);

        let app = create_app(mock_builder());
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();

        let path_wrap = ModifyWrap {
            name: "Test".to_string(),
            path: profile_path.clone(),
        };
        let serialized_payload = serde_json::to_string(&path_wrap).unwrap();
        let invoke_body = tauri::ipc::InvokeBody::Json(serialized_payload.parse().unwrap());

        let res = tauri::test::get_ipc_response(
            &webview,
            tauri::webview::InvokeRequest {
                cmd: "add_profile".into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: "http://tauri.localhost".parse().unwrap(),
                body: invoke_body,
                headers: Default::default(),
            },
        );

        assert!(res.is_ok());
        assert_eq!(
            res.clone().unwrap().deserialize::<Vec<Profile>>().unwrap()[0].currently,
            false
        );
        assert_eq!(
            res.unwrap().deserialize::<Vec<Profile>>().unwrap()[1].currently,
            true
        );
    }

    #[test]
    fn test_remove_profile() {
        let tmp_dir = tempdir().unwrap();
        let profile_path = tmp_dir.path().join("profile.json");

        let mut profiles: Vec<Profile> = Vec::new();
        let profile = Profile {
            name: "Default".to_string(),
            mods: Vec::new(),
            currently: true,
        };
        let test_profile = Profile {
            name: "Test".to_string(),
            mods: Vec::new(),
            currently: false,
        };
        profiles.push(profile.clone());
        profiles.push(test_profile.clone());

        save_profiles(&profiles, &profile_path);

        let app = create_app(mock_builder());
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();

        let path_wrap = ModifyWrap {
            name: "Test".to_string(),
            path: profile_path.clone(),
        };
        let serialized_payload = serde_json::to_string(&path_wrap).unwrap();
        let invoke_body = tauri::ipc::InvokeBody::Json(serialized_payload.parse().unwrap());

        let res = tauri::test::get_ipc_response(
            &webview,
            tauri::webview::InvokeRequest {
                cmd: "remove_profile".into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: "http://tauri.localhost".parse().unwrap(),
                body: invoke_body,
                headers: Default::default(),
            },
        );

        assert!(res.is_ok());
        assert_eq!(
            res.clone()
                .unwrap()
                .deserialize::<Vec<Profile>>()
                .unwrap()
                .iter()
                .count(),
            1
        );
    }

    #[test]
    fn test_modify_profile() {
        let tmp_dir = tempdir().unwrap();
        let profile_path = tmp_dir.path().join("profile.json");

        let mut profiles: Vec<Profile> = Vec::new();
        let profile = Profile {
            name: "Default".to_string(),
            mods: Vec::new(),
            currently: true,
        };
        let test_profile = Profile {
            name: "Test".to_string(),
            mods: Vec::new(),
            currently: false,
        };
        profiles.push(profile.clone());
        profiles.push(test_profile.clone());

        save_profiles(&profiles, &profile_path);

        let app = create_app(mock_builder());
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();

        let path_wrap = ModifyNameWrap {
            name: "Test".to_string(),
            new_name: "Test 2".to_string(),
            path: profile_path.clone(),
        };
        let serialized_payload = serde_json::to_string(&path_wrap).unwrap();
        let invoke_body = tauri::ipc::InvokeBody::Json(serialized_payload.parse().unwrap());

        let res = tauri::test::get_ipc_response(
            &webview,
            tauri::webview::InvokeRequest {
                cmd: "modify_profile".into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: "http://tauri.localhost".parse().unwrap(),
                body: invoke_body,
                headers: Default::default(),
            },
        );

        assert!(res.is_ok());
        assert_eq!(
            res.unwrap().deserialize::<Vec<Profile>>().unwrap()[1].name,
            "Test 2"
        );
    }

    #[test]
    fn test_change_profile_mods() {
        let tmp_dir = tempdir().unwrap();
        let profile_path = tmp_dir.path().join("profile.json");

        let mut profiles: Vec<Profile> = Vec::new();
        let profile = Profile {
            name: "Default".to_string(),
            mods: Vec::new(),
            currently: true,
        };
        let test_profile = Profile {
            name: "Test".to_string(),
            mods: Vec::new(),
            currently: false,
        };
        profiles.push(profile.clone());
        profiles.push(test_profile.clone());

        save_profiles(&profiles, &profile_path);

        let app = create_app(mock_builder());
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();

        let new_mod = ModInfo {
            name: "Test Mod".to_string(),
            summary: Some("Test Summary".to_string()),
            description: Some("Test Description".to_string()),
            picture_url: None,
            mod_downloads: 0,
            mod_unique_downloads: 0,
            uid: 0,
            mod_id: 0,
            game_id: 1303,
            allow_rating: false,
            domain_name: "stardewvalley".to_owned(),
            category_id: 0,
            version: "1.0.0".to_string(),
            endorsement_count: 0,
            created_timestamp: 0,
            created_time: "".to_owned(),
            updated_timestamp: 0,
            updated_time: "".to_owned(),
            author: "Test Author".to_string(),
            uploaded_by: "Test Author".to_string(),
            uploaded_users_profile_url: "".to_owned(),
            contains_adult_content: false,
            status: "".to_owned(),
            available: true,
            unique_id: Some("1234".to_string()),
            more_info: None,
            dependencies: None,
            group: None,
        };

        let mod_wrap = ModWrap {
            name: "Test".to_string(),
            path: profile_path.clone(),
            mods: vec![new_mod.clone()],
        };
        let serialized_payload = serde_json::to_string(&mod_wrap).unwrap();
        let invoke_body = tauri::ipc::InvokeBody::Json(serialized_payload.parse().unwrap());

        let res = tauri::test::get_ipc_response(
            &webview,
            tauri::webview::InvokeRequest {
                cmd: "change_profile_mods".into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: "http://tauri.localhost".parse().unwrap(),
                body: invoke_body,
                headers: Default::default(),
            },
        );

        let loaded_profiles = get_profiles(profile_path.clone());

        assert!(res.is_ok());
        assert_eq!(loaded_profiles[1].mods.len(), 1);
    }
}
