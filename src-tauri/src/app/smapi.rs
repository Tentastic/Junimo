use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Command, Stdio};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::{command, Manager, WebviewUrl};
use walkdir::WalkDir;
use zip::ZipArchive;
use crate::app::utility::{paths, zips};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SmapiProcess {
    pub started: bool,
    pub download_progress: u64,
    pub size: u64,
    pub download_finished: bool,
    pub installation_started: bool,
    pub installation_finished: bool,
}

impl SmapiProcess {
    pub fn new(started: bool, size: u64, installation_started: bool) -> Self {
        SmapiProcess {
            started,
            download_progress: 0,
            size,
            download_finished: false,
            installation_started,
            installation_finished: false,
        }
    }
}

#[command]
pub async fn open_smapi(handle: tauri::AppHandle) {
    #[cfg(target_os = "windows")]
    tauri::WebviewWindowBuilder::new(&handle, "Smapi", WebviewUrl::App("/smapi".into()))
        .title("Smapi")
        .resizable(false)
        .maximizable(false)
        .inner_size(700.0, 350.0)
        .transparent(true)
        .build()
        .unwrap();

    #[cfg(target_os = "unix")]
    tauri::WebviewWindowBuilder::new(&handle, "Smapi", WebviewUrl::App("/smapi".into()))
        .title("Smapi")
        .resizable(false)
        .maximizable(false)
        .inner_size(600.0, 350.0)
        .build()
        .unwrap();
}

#[command]
pub async fn download_smapi(handle: tauri::AppHandle, link: String) -> Result<(), String> {
    let client = reqwest::Client::new();
    let mut response = client.get(link).send().await.unwrap();
    if !response.status().is_success() {
        panic!("Received non-successful status code: {}", response.status());
    }

    let total_size = response
        .content_length()
        .expect("Failed to get content length");

    let mut progress: SmapiProcess = SmapiProcess::new(true, total_size, false);
    handle.emit("smapi_progress", &progress).unwrap();

    let zip_path = "smapi.zip".to_string();
    let mut temp_path = paths::temp_path();
    temp_path.push(&zip_path);
    let mut file = File::create(&temp_path).unwrap();

    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();


    while let Some(item) = stream.next().await {
        let chunk = item.expect("Failed to read chunk");
        file.write_all(&chunk).unwrap();
        let new = downloaded + (chunk.len() as u64);
        progress.download_progress = new;
        downloaded = new;

        // Emit the updated download object to the frontend
        handle.emit("smapi_progress", &progress).unwrap();
    }

    progress.download_progress = progress.size;
    progress.download_finished = true;
    handle.emit("smapi_progress", &progress).unwrap();

    // Flushes and deletes the download object
    file.flush().unwrap();
    drop(file);

    install_smapi().await;

    Ok(())
}

pub async fn install_smapi() {
    let mut temp_path = paths::temp_path();
    temp_path = temp_path.join("smapi.zip");

    let mut desination = paths::appdata_path();
    desination = desination.join("smapi");

    let file = File::open(&temp_path).unwrap();
    let zip_archive = zip::ZipArchive::new(file).unwrap();

    let unpack_result = zips::unpack_smapi(zip_archive, &desination);

    #[cfg(target_os = "windows")]
    let dat_path = desination.join(unpack_result.unwrap()).join("internal").join("windows").join("install.dat");
    #[cfg(target_os = "linux")]
    let dat_path = desination.join(unpack_result.unwrap()).join("internal").join("linux").join("install.dat");
    #[cfg(target_os = "macos")]
    let dat_path = desination.join(unpack_result.unwrap()).join("internal").join("macOS").join("install.dat");

    let file = File::open(&dat_path).unwrap();
    let buf_reader = BufReader::new(file);

    let archive = ZipArchive::new(buf_reader).unwrap();
    let unpack_result = zips::unpack_smapi(archive, &paths::get_game_path());

    match unpack_result {
        Ok(_) => {

        }
        Err(e) => {
            println!("Failed to unpack smapi: {}", e);
        }
    }
}