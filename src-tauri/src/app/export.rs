use std::error::Error;
use std::fs::File;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::{fs, io, thread};

use rfd::FileDialog;
use tauri::{command, Manager, Runtime, WebviewUrl, Window};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::app::models::mod_info::ModInfo;
use crate::app::profiles::Profile;
use crate::app::utility::{paths, zips};
use crate::app::{console, mods, profiles};

#[command]
pub async fn open_export<R: Runtime>(handle: tauri::AppHandle<R>) {
    #[cfg(target_os = "windows")]
    tauri::WebviewWindowBuilder::new(&handle, "Exporter", WebviewUrl::App("/exporter".into()))
        .title("Export")
        .min_inner_size(600.0, 350.0)
        .inner_size(600.0, 350.0)
        .transparent(true)
        .build()
        .unwrap();

    #[cfg(target_os = "unix")]
    tauri::WebviewWindowBuilder::new(&handle, "Exporter", WebviewUrl::App("/exporter".into()))
        .title("Export")
        .min_inner_size(600.0, 350.0)
        .inner_size(600.0, 350.0)
        .build()
        .unwrap();
}

#[command]
pub fn select_export_dir() -> String {
    let file = FileDialog::new().set_directory(".").pick_folder();

    if let Some(path) = file {
        path.to_string_lossy().to_string()
    } else {
        "".to_string()
    }
}

#[command]
pub fn export_profile<R: Runtime>(
    window: Window,
    handle: tauri::AppHandle<R>,
    name: String,
    path: String,
) -> bool {
    let handle_clone = handle.clone();
    let window_clone = window.clone();
    let name_clone = name.clone();
    let path_clone = path.clone();

    thread::spawn(move || {
        if name == "All Profiles" {
            export_all(handle_clone, path_clone);
            window_clone.close().unwrap();
        } else {
            export_one(handle_clone, name_clone, path_clone);
            window_clone.close().unwrap();
        }
    });
    true
}

fn export(
    zip_path: &PathBuf,
    mods: &Vec<ModInfo>,
    profile_path: &PathBuf,
    mod_path: Option<PathBuf>,
) -> Result<(), String> {
    let zip_file = File::create(zip_path);

    if zip_file.is_err() {
        return Err(zip_file.err().unwrap().to_string());
    }

    let zip_file = zip_file.unwrap();
    let mut zip = ZipWriter::new(zip_file);

    for mod_info in mods {
        let mut mod_path = paths::mod_path();
        let mod_paths = &mod_path.join(&mod_info.name);
        let mod_path_dot = &mod_path.join(format!(".{}", &mod_info.name));

        if mod_paths.exists() {
            zips::zip_mods(&mut zip, mod_paths);
        } else if mod_path_dot.exists() {
            zips::zip_mods(&mut zip, mod_path_dot);
        }
    }

    let to_profile_path = PathBuf::from("profile.json");
    let to_mods_path = PathBuf::from("mods.json");
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    zip.start_file(to_profile_path.to_string_lossy(), options)
        .unwrap();
    let mut profile_file = File::open(profile_path).unwrap();
    io::copy(&mut profile_file, &mut zip).unwrap();

    if mod_path.is_some() {
        zip.start_file(to_mods_path.to_string_lossy(), options)
            .unwrap();
        let mut mods_file = File::open(mod_path.unwrap()).unwrap();
        io::copy(&mut mods_file, &mut zip).unwrap();
    }

    return match zip.finish() {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    };
}

fn export_one<R: Runtime>(handle: tauri::AppHandle<R>, name: String, path: String) {
    let profiles = profiles::get_profiles(paths::profile_path());
    let mut profile: Profile = Profile {
        name: "None".to_string(),
        mods: vec![],
        currently: false,
    };

    for prof in profiles {
        if prof.name == name {
            profile = prof;
        }
    }

    let export_path = Path::new(&path).join(format!("export_{}.zip", profile.name));

    let profiles: Vec<Profile> = vec![profile.clone()];
    let profile_json = serde_json::to_string(&profiles).unwrap();
    let mut temp_path = paths::temp_path().join(&name);
    fs::create_dir_all(&temp_path).unwrap();
    let temp_file_path = temp_path.join("profile.json");
    fs::write(&temp_file_path, profile_json).unwrap();

    match export(&export_path, &profile.mods, &temp_file_path, None) {
        Ok(_) => {
            fs::remove_dir_all(&temp_path).unwrap();
            console::add_line(
                &handle,
                format!(
                    "<span class=\"console-green\">[Junimo] Exported profile {}</span>",
                    profile.name
                ),
            );
        }
        Err(e) => {
            console::add_line(
                &handle,
                format!(
                    "<span class=\"console-red\">[Junimo] Error exporting all profiles and mods: {}</span>",
                    e
                ),
            );
        }
    }
}

fn export_all<R: Runtime>(handle: tauri::AppHandle<R>, path: String) {
    let export_path = Path::new(&path).join("export_all.zip");
    let profile_file = paths::appdata_path().join("profile.json");
    let mods_file = paths::appdata_path().join("mods.json");

    match export(
        &export_path,
        &mods::get_all_mods(),
        &profile_file,
        Some(mods_file),
    ) {
        Ok(_) => {
            console::add_line(
                &handle,
                "<span class=\"console-green\">[Junimo] Exported all profiles and mods</span>"
                    .to_string(),
            );
        }
        Err(e) => {
            console::add_line(
                &handle,
                format!(
                    "<span class=\"console-red\">[Junimo] Error exporting all profiles and mods: {}</span>",
                    e
                ),
            );
        }
    };
}

#[cfg(test)]
mod tests {
    use tauri::test::mock_builder;
    use tempfile::tempdir;

    use crate::app::app_state::AppState;

    use super::*;

    fn create_app<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::App<R> {
        let (app_state, rx) = AppState::new();

        builder
            .invoke_handler(tauri::generate_handler![open_export, close_export])
            .manage(app_state.clone())
            // remove the string argument to use your app's config file
            .build(tauri::generate_context!())
            .expect("failed to build app")
    }
    #[test]
    fn test_open_export() {
        let app = create_app(mock_builder());
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();

        let res = tauri::test::get_ipc_response(
            &webview,
            tauri::webview::InvokeRequest {
                cmd: "open_export".into(),
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
    fn test_close_export() {
        let app = create_app(mock_builder());
        let webview = tauri::WebviewWindowBuilder::new(&app, "Export", Default::default())
            .build()
            .unwrap();

        let res = tauri::test::get_ipc_response(
            &webview,
            tauri::webview::InvokeRequest {
                cmd: "close_export".into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: "http://tauri.localhost".parse().unwrap(),
                body: tauri::ipc::InvokeBody::default(),
                headers: Default::default(),
            },
        );
        assert!(res.is_ok());
    }
}
