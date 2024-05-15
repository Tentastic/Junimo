use crate::app::profiles::Profile;
use crate::app::utility::{paths, zips};
use crate::app::{console, mod_installation, mods, profiles};
use rfd::FileDialog;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{command, Runtime, WebviewUrl, Window};

#[command]
pub async fn open_export<R: Runtime>(handle: tauri::AppHandle<R>) {
    tauri::WebviewWindowBuilder::new(&handle, "Exporter", WebviewUrl::App("/exporter".into()))
        .title("Export")
        .min_inner_size(600.0, 350.0)
        .inner_size(600.0, 350.0)
        .transparent(true)
        .build()
        .unwrap();
}

#[command]
pub async fn close_export<R: Runtime>(window: Window<R>) {
    window.close().unwrap();
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
pub fn export_profile<R: Runtime>(handle: tauri::AppHandle<R>, name: &str, path: &str) -> bool {
    if name == "All Profiles" {
        export_all(handle, path);
        true
    } else {
        export_one(handle, name, path);
        true
    }
}

fn export_one<R: Runtime>(handle: tauri::AppHandle<R>, name: &str, path: &str) {
    let profiles = profiles::get_profiles(paths::profile_path());
    let mut profile: Profile = Profile {
        name: "".to_string(),
        mods: vec![],
        currently: false,
    };

    for prof in profiles {
        if prof.name == name {
            profile = prof;
        }
    }

    let export_path = format!("export_{}", profile.name);

    let mut export_dir = paths::temp_path();
    export_dir.push(&export_path);
    fs::create_dir_all(&export_dir).unwrap();

    for mod_info in &profile.mods {
        let mut mod_path = paths::mod_path();
        mod_path.push(format!("{}.zip", mod_info.name));
        let mut export_mod_path = export_dir.clone();
        export_mod_path.push("mods");
        fs::create_dir_all(&export_mod_path).unwrap();
        export_mod_path.push(format!("{}.zip", mod_info.name));
        fs::copy(&mod_path, &export_mod_path).unwrap();
    }

    let export_zip_path = Path::new(path).join(format!("export_{}.zip", &profile.name));

    let profiles: Vec<Profile> = vec![profile];
    let profile_json = serde_json::to_string(&profiles).unwrap();
    let profile_json_path = export_dir.clone();
    fs::write(profile_json_path.join("profile.json"), profile_json).unwrap();

    zips::zip_directory(&export_dir, &export_zip_path).unwrap();
    fs::remove_dir_all(&export_dir).unwrap();
    console::add_line(
        &handle,
        "<span class=\"console-green\">[Junimo] Exported profile</span>".to_string(),
    );
}

fn export_all<R: Runtime>(handle: tauri::AppHandle<R>, path: &str) {
    let mut export_dir = paths::temp_path();
    export_dir.push("export_all");
    fs::create_dir_all(&export_dir).unwrap();

    for mod_info in &mods::get_all_mods() {
        let mut mod_path = paths::mod_path();
        mod_path.push(format!("{}.zip", mod_info.name));
        let mut export_mod_path = export_dir.clone();
        export_mod_path.push("mods");
        fs::create_dir_all(&export_mod_path).unwrap();
        export_mod_path.push(format!("{}.zip", mod_info.name));
        if !mod_path.exists() {
            continue;
        }
        fs::copy(&mod_path, &export_mod_path).unwrap();
    }

    let export_zip_path = Path::new(path).join("export_all.zip");

    let profile_file = paths::appdata_path().join("profile.json");
    let mods_file = paths::appdata_path().join("mods.json");
    fs::copy(&profile_file, export_dir.join("profile.json")).unwrap();
    fs::copy(&mods_file, export_dir.join("mods.json")).unwrap();

    zips::zip_directory(&export_dir, &export_zip_path).unwrap();
    fs::remove_dir_all(&export_dir).unwrap();
    console::add_line(
        &handle,
        "<span class=\"console-green\">[Junimo] Exported all profiles and mods</span>".to_string(),
    );
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
            .invoke_handler(tauri::generate_handler![
                open_export,
                close_export
            ])
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