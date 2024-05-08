use std::{fs, io, thread};
use std::fs::File;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};
use zip::ZipArchive;
use crate::app::{console, mods};
use crate::app::utility::{paths, zips};

pub fn start_installation(app_handle: AppHandle, path: &PathBuf) {
    let cloned_handle = app_handle.clone();
    let path_clone = path.clone();
    thread::spawn(move || {
        console::add_line(&app_handle, "<span class=\"console-green\">[Junimo] Started to add mod</span>".to_string());

        let zip_file_path = path_clone;
        let output_folder_path = paths::temp_path();
        let to_path = format!("{}/{}", output_folder_path.display(), zip_file_path.file_name().unwrap().to_str().unwrap());
        fs::copy(&zip_file_path, &to_path).unwrap();
        let copied_zip_file = File::open(&to_path).unwrap();
        let zip_archive = ZipArchive::new(copied_zip_file).unwrap();

        extract_mods(&cloned_handle, zip_archive, &output_folder_path);
    });
}

fn calculate_percentage(value: &usize, max_value: &usize) -> usize {
    if max_value == &0 {
        return 0;
    }
    let scaled_value = value * 10;
    let result = scaled_value / max_value;
    let remainder = scaled_value % max_value;
    if remainder >= max_value / 2 {
        return result + 1;
    }
    result
}

fn list_dirs_in_directory(path: &Path) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = Vec::new();
    if path.is_dir() {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                paths.push(path);
            }
        }
    }
    paths
}

fn install_progress(amount: &usize, max: &usize) -> String {
    let mut new_console_output = "<span class=\"console-gray\">[Junimo] Install progress [".to_string();
    let calculated_percentage = calculate_percentage(amount, max);
    for rin in 0..calculated_percentage {
        new_console_output = format!("{}█", new_console_output);
    }
    for rin in calculated_percentage..10 {
        new_console_output = format!("{}⠀", new_console_output);
    }
    format!("{}]</span>", new_console_output)
}

fn extract_mods<R: io::Read + io::Seek>(app_handle: &AppHandle, mut archive: ZipArchive<R>, destination: &Path) {
    let mut main_dir = "".to_string();
    let mut has_manifest = false;
    let max = archive.len();
    let mut depth = 0;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let split = file.name().split('/').collect::<Vec<&str>>();

        if !file.name().contains("manifest.json") {
            depth = split.len();
            break;
        }
    }

    console::add_line(&app_handle, install_progress(&0, &max).to_string());

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let split = file.name().split('/').collect::<Vec<&str>>();

        if split[0] != main_dir {
            main_dir = split[0].to_string();
        }

        let outpath = match file.enclosed_name() {
            Some(path) => destination.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            if !outpath.exists() {
                fs::create_dir_all(&outpath).unwrap();
            }
        } else {
            if let Some(parent) = outpath.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent).unwrap();
                }
            }
            let mut outfile = File::create(&outpath).unwrap();
            io::copy(&mut file, &mut outfile).unwrap();

            if outpath.to_string_lossy().contains("manifest.json") {
                has_manifest = true;
            }
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode))?;
            }
        }

        console::modify_line(&app_handle, install_progress(&i, &max).to_string());
    }

    if !has_manifest {
        console::add_line(&app_handle, "<span class=\"console-red\">[Junimo] Not a valid Stardew Valley mod!</span>".to_string());
        &app_handle.emit_all("reload", false).unwrap();
        return;
    }

    if depth > 3 {
        let outpath = destination.join(main_dir);
        let directories = list_dirs_in_directory(&outpath);
        copy_into_mod(directories, &outpath);
    }
    else {
        let outpath = destination.join(main_dir);
        let dir = outpath.clone();
        let directories: Vec<PathBuf> = vec![dir];
        copy_into_mod(directories, &outpath);
    }

    console::modify_line(&app_handle, format!("<span class=\"console-green\">[Junimo] Mod installed</span>"));
    &app_handle.emit_all("reload", false).unwrap();
}

fn copy_into_mod(directories: Vec<PathBuf>, outpath: &PathBuf) {
    for dir in directories {
        let mut new_dir_name = "".to_string();
        for entry in fs::read_dir(&dir).unwrap() {
            match entry {
                Ok(entry) => {
                    if entry.file_name().to_string_lossy().contains("manifest.json") {
                        new_dir_name = mods::add_mod_through_manifest(&entry.path());
                    }
                },
                Err(_) => {
                    continue;
                }
            }
        }

        let mut renamed_dir = outpath.clone();
        renamed_dir.push(&new_dir_name);
        fs::rename(&dir, &renamed_dir).unwrap();

        let mut new_zip_path = paths::mod_path();
        new_zip_path.push(&renamed_dir);

        let new_path = format!("{}/{}.zip", paths::mod_path().to_string_lossy(), &new_dir_name);
        zips::zip_directory(&renamed_dir, Path::new(&new_path)).unwrap();
    }
}