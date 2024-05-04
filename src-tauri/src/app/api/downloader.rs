use std::fs;
use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Mutex};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, command, Manager, State};
use crate::app::api::{downloader, mods_api, nexuswebsocket};
use crate::app::app_state::AppState;
use crate::app::mods::{get_mods, get_path, ModInfo};

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
    pub current: u64
}

#[command]
pub fn stop_download(app_state: State<'_, AppState>) {
    let mut signal = app_state.stop_download.lock().unwrap();
    *signal = true;
}

pub async fn start_download(app_handle: &AppHandle, url_str: &str, app_state: AppState) {
    println!("Scheme: {}", url_str);
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

        let mut mod_list = get_mods();

        if mod_list.iter().any(|mod_info| mod_info.name == infos.name && mod_info.version == infos.version) {
            return;
        }
        else if mod_list.iter().any(|mod_info| mod_info.name == infos.name && mod_info.version != infos.version) {
            let index = mod_list.iter().position(|mod_info| mod_info.name == infos.name).unwrap();
            mod_list[index] = infos.clone();
            crate::app::mods::save_mods(mod_list);
        }
        else {
            mod_list.push(infos.clone());
            crate::app::mods::save_mods(mod_list);
        }

        download(app_handle, &paths[0].URI, infos, app_state).await;
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
        current: 0
    };

    app_handle.emit_all("download", &download).unwrap();

    let zip_path = format!("{}.zip", infos.name);
    let mut mods_path = std::env::temp_dir();
    mods_path.push("Junimo");
    fs::create_dir_all(&mods_path).unwrap();
    mods_path.push(&zip_path);

    let mut file = File::create(&mods_path).unwrap();

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

    if (&download.size == &download.current) {
        app_handle.emit_all("downloadfinished", &download).unwrap();
        let mut path = get_path();
        path.push(&zip_path);
        fs::rename(mods_path, path).unwrap();
    }
    else {
        fs::remove_file (mods_path).unwrap();
        app_handle.emit_all("downloadfailed", &download).unwrap();
    }

    let console_output = format!("<span class=\"console-green\">Downloaded</span>: {}", infos.name);
    app_handle.emit_all("console", console_output).unwrap();
}