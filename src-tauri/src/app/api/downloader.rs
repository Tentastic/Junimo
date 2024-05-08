use std::{fs, thread};
use std::fs::File;
use std::io::Write;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, command, Manager, State};
use crate::app::api::{mods_api, nexuswebsocket};
use crate::app::app_state::AppState;
use crate::app::{console, mods};
use crate::app::mods::{get_all_mods, ModInfo};
use crate::app::utility::paths;

#[derive(Serialize, Deserialize, Debug)]
struct DownloadPaths {
    name: String,
    short_name: String,
    URI: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Download {
    pub name: String,
    pub size: u64,
    pub current: u64,
    pub aborted: bool,
    pub finished: bool
}

#[command]
pub fn stop_download(app_state: State<'_, AppState>) {
    let mut signal = app_state.stop_download.lock().unwrap();
    *signal = true;
}

pub async fn start_download(app_handle: &AppHandle, url_str: &str, app_state: AppState) {
    let key = nexuswebsocket::load_key();

    let client = reqwest::Client::new();
    let res = client.get(mods_api::get_url(url_str))
        .header("accept", "application/json")
        .header("apikey", key)
        .send()
        .await
        .unwrap();

    if res.status().is_success() {
        let body = res.text().await.unwrap();
        let body_str = body.as_str();
        let paths: Vec<DownloadPaths> = serde_json::from_str(body_str).unwrap();

        let infos = mods_api::get_infos(url_str).await.unwrap();

        mods::insert_mod_info(&infos);

        let handle_clone = app_handle.clone();
        download(&handle_clone, &paths[0].URI, infos, app_state).await;
    }
}

async fn download(app_handle: &AppHandle, url_str: &str, infos: ModInfo, app_state: AppState) {
    let client = reqwest::Client::new();
    let mut response = client.get(url_str).send().await.unwrap();
    if !response.status().is_success() {
        panic!("Received non-successful status code: {}", response.status());
    }

    let total_size = response
        .content_length()
        .expect("Failed to get content length");

    let mut download: Download = Download {
        name: infos.name.to_owned(),
        size: total_size,
        current: 0,
        aborted: false,
        finished: false
    };

    app_handle.emit_all("download", &download).unwrap();

    let zip_path = format!("{}.zip", infos.name);
    let mut temp_path = paths::temp_path();
    temp_path.push(&zip_path);

    let mut file = File::create(&temp_path).unwrap();

    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        let should_stop = app_state.stop_download.lock().unwrap();
        if *should_stop {
            break;
        }

        let chunk = item.expect("Failed to read chunk");
        file.write_all(&chunk).unwrap();
        let new = downloaded + (chunk.len() as u64);
        download.current = new;
        downloaded = new;

        app_handle.emit_all("download", &download).unwrap();
    }

    if &download.size == &download.current {
        app_handle.emit_all("downloadfinished", &download).unwrap();
        let mut path = paths::mod_path();
        path.push(&zip_path);
        fs::rename(temp_path, &path).unwrap();
        download.finished = true;
        mods::unpack_manifest(&path, &infos.name);
        app_handle.emit_all("download", &download).unwrap();
        let console_output = format!("<span class=\"console-green\">Downloaded</span>: {}", infos.name);
        console::add_line(&app_handle, console_output);
    }
    else {
        let mut mod_list = get_all_mods();
        mod_list.retain(|mod_info| mod_info.name != infos.name);
        mods::save_mods(mod_list);
        fs::remove_file (temp_path).unwrap();
        download.aborted = true;
        app_handle.emit_all("download", &download).unwrap();
    }
}