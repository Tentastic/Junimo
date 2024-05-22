use crate::app::api::{compatibility, mods_api, nexuswebsocket};
use crate::app::app_state::AppState;
use crate::app::mods::{get_all_mods, ModInfo};
use crate::app::utility::{paths, zips};
use crate::app::{console, mod_installation, mods};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::{fs, thread};
use tauri::{command, AppHandle, Manager, State};

#[derive(Serialize, Deserialize, Debug)]
struct DownloadPaths {
    name: String,
    short_name: String,
    URI: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Download {
    pub name: String,
    pub size: u64,
    pub current: u64,
    pub aborted: bool,
    pub finished: bool,
}

impl Download {
    pub fn new(name: String, size: u64) -> Self {
        Download {
            name,
            size,
            current: 0,
            aborted: false,
            finished: false,
        }
    }
}

#[command]
pub fn stop_download(app_state: State<'_, AppState>) {
    let mut signal = app_state.stop_download.lock().unwrap();
    *signal = true;
}

/// Starts the download of a mod
///
/// * `app_handle` - Tauri Handle
/// * `url_str` - The url of the mod
/// * `app_state` - The state of the app
pub async fn start_download(app_handle: &AppHandle, url_str: &str, app_state: AppState) {
    // Loads api key from our config file
    let key = nexuswebsocket::load_key();
    if key.is_empty() {
        let console_output = "<span class=\"console-red\">No NexusMods API Key found. Please provide a key in settings.</span>".to_string();
        console::add_line(&app_handle, console_output);
        return;
    }

    // Sends a request to the NexusMods API to get the download link
    let client = reqwest::Client::new();
    let res = client
        .get(mods_api::get_download_link(url_str))
        .header("accept", "application/json")
        .header("apikey", key)
        .send()
        .await
        .unwrap();

    if res.status().is_success() {
        // Parses the response to get the download links
        let body = res.text().await.unwrap();
        let body_str = body.as_str();
        let paths: Vec<DownloadPaths> = serde_json::from_str(body_str).unwrap();

        // Gets infos about the mod
        let infos = mods_api::get_infos(url_str).await.unwrap();

        let handle_clone = app_handle.clone();
        download(&handle_clone, &paths[0].URI, infos, app_state).await;
    }
    else {
        println!("Failed to get download link {}", res.status());
    }
}

/// Downloads the mod
///
/// * `app_handle` - Tauri Handle
/// * `url_str` - Download url of the mod
/// * `infos` - Infos about the mod
/// * `app_state` - The state of the app
async fn download(app_handle: &AppHandle, url_str: &str, infos: ModInfo, app_state: AppState) {
    // Sends a request to the download url
    let client = reqwest::Client::new();
    let mut response = client.get(url_str).send().await.unwrap();
    if !response.status().is_success() {
        panic!("Received non-successful status code: {}", response.status());
    }

    // Gets the total size of the download
    let total_size = response
        .content_length()
        .expect("Failed to get content length");

    // Creates a download object and make it visible to the user by emitting it to the frontend
    let mut download: Download = Download::new(infos.name.to_owned(), total_size);
    app_handle.emit("download", &download).unwrap();

    // Creates a zip file with the name of the mod in the temp folder
    let zip_path = format!("{}.zip", infos.name);
    let mut temp_path = paths::temp_path();
    temp_path.push(&zip_path);
    let mut file = File::create(&temp_path).unwrap();

    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    // Downloads the mod chunk by chunk
    while let Some(item) = stream.next().await {
        // Check if the user wants to stop the download
        let should_stop = app_state.stop_download.lock().unwrap();
        if *should_stop {
            break;
        }

        let chunk = item.expect("Failed to read chunk");
        file.write_all(&chunk).unwrap();
        let new = downloaded + (chunk.len() as u64);
        download.current = new;
        downloaded = new;

        // Emit the updated download object to the frontend
        app_handle.emit("download", &download).unwrap();
    }

    // Flushes and deletes the download object
    file.flush().unwrap();
    drop(file);

    // Check if the download was finished or aborted
    if &download.size == &download.current {
        // Show the user that the download was finished
        app_handle.emit("downloadfinished", &download).unwrap();
        download.finished = true;
        app_handle.emit("download", &download).unwrap();
        let console_output = format!(
            "<span class=\"console-green\">Downloaded</span>: {}",
            infos.name
        );
        console::add_line(&app_handle, console_output);

        // Start installing the mod through the downloaded zip file
        let _ = mod_installation::start_installation(app_handle.clone(), &temp_path).await;
        match fs::remove_file(&temp_path) {
            Ok(_) => (),
            Err(_) => ()
        }
    }
    else {
        // Show the user that the download was aborted
        let mut mod_list = get_all_mods();
        mod_list.retain(|mod_info| mod_info.name != infos.name);
        mods::save_mods(mod_list);
        fs::remove_file(temp_path).unwrap();
        download.aborted = true;
        app_handle.emit("download", &download).unwrap();
    }
}
