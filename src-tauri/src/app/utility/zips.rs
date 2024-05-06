use std::{fs, io};
use std::fs::File;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};
use walkdir::WalkDir;
use zip::write::{FileOptions, SimpleFileOptions};
use zip::{ZipArchive, ZipWriter};
use crate::app::{console, mods};
use crate::app::utility::paths;

pub fn extract_manifest<R: io::Read + io::Seek>(mut archive: ZipArchive<R>, destination: &Path, name: &str) -> zip::result::ZipResult<()> {
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        if !file.name().contains("manifest.json") {
            continue;
        }
        let split = file.name().split('/').collect::<Vec<&str>>();
        let new_path = format!("{}/{}", name, split[split.len() - 1]);
        let outpath = destination.join(new_path);


        if file.name().ends_with('/') {
            if !outpath.exists() {
                fs::create_dir_all(&outpath)?;
            }
        } else {
            let mut mod_path = paths::mod_path();
            mod_path.push(name);
            if !mod_path.exists() {
                fs::create_dir_all(&mod_path)?;
            }
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode))?;
            }
        }
    }
    Ok(())
}

fn calculate_percentage(value: usize, max_value: usize) -> usize {
    if max_value == 0 {
        return 0;
    }
    let scaled_value = value * 10;
    let result = scaled_value / max_value;
    let remainder = scaled_value % max_value;
    if remainder >= max_value / 2 { // Simple rule to round up if the remainder is half the max_value or more
        return result + 1;
    }
    result
}

pub fn extract_sub_folders<R: io::Read + io::Seek>(app_handle: &AppHandle, mut archive: ZipArchive<R>, destination: &Path) -> zip::result::ZipResult<()> {
    let mut dir_paths: Vec<String> = Vec::new();
    let mut dependencies: Vec<mods::Dependency> = Vec::new();
    let mut already_installed: Vec<String> = Vec::new();
    let mut save_path: PathBuf = PathBuf::new();
    let mut mod_name: String = "".to_string();
    let mut depth = 0;
    let max = archive.len();

    let mut console_output = "<span class=\"console-gray\">[Junimo] Install progress [".to_string();
    for i in 0..10 {
       console_output = format!("{}⠀", console_output);
    }
    console_output = format!("{}]</span>", console_output);
    console::add_line(&app_handle, console_output.to_string());

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        if !file.name().contains("manifest.json") {
            continue;
        }
        let split = file.name().split('/').collect::<Vec<&str>>();
        depth = split.len();
        break;
    }

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let split = file.name().split('/').collect::<Vec<&str>>();
        let split_clone = split.clone();
        mod_name = split_clone[0].to_string();

        let mut outpath = paths::temp_path();

        if split.len() > depth {
            for j in 0..split.len() - 1 {
                outpath = outpath.join(split[j]);
            }
        }
        else {
            continue;
        }
        //outpath = outpath.join(file.enclosed_name().unwrap().to_string_lossy().to_string());

        if !&dir_paths.contains(&split[0].to_string()) {
            let mut new_path: Vec<String> = Vec::new();
            new_path.push(split[0].to_string());
            dir_paths = new_path;
        }

        if file.name().ends_with('/') {
            if !outpath.exists() {
                fs::create_dir_all(&outpath)?;
            }
        } else {
            if let Some(parent) = outpath.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;

            if outpath.to_string_lossy().contains("manifest.json") {
                save_path = outpath.clone();

                let mod_manifest = mods::get_manifest(&outpath);
                already_installed.push(mod_manifest.clone().unique_id.clone());

                let found_dependencies = mod_manifest.clone().dependencies;
                let found_content_pack = mod_manifest.clone().content_pack;

                match found_dependencies {
                    Some(dependency) => {
                        for dep in &dependency {
                            dependencies.push(dep.clone());
                        }
                    },
                    None => {
                    }
                }

                match found_content_pack {
                    Some(dependency) => {
                        dependencies.push(dependency.clone());
                    },
                    None => {
                    }
                }
            }
        }

        let mut new_console_output = "<span class=\"console-gray\">[Junimo] Install progress [".to_string();
        let calculated_percentage = calculate_percentage(i, max);
        for rin in 0..calculated_percentage {
            new_console_output = format!("{}█", new_console_output);
        }
        for rin in calculated_percentage..10 {
            new_console_output = format!("{}⠀", new_console_output);
        }
        new_console_output = format!("{}]</span>", new_console_output);
        console::modify_line(&app_handle, new_console_output.to_string());

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode))?;
            }
        }
    }

    for installed in already_installed {
        dependencies.retain(|dep| dep.unique_id != installed);
    }

    mods::add_mod_through_manifest(&save_path, &mod_name, dependencies);
    println!("Length: {}", &dir_paths.len());
    for dir in dir_paths {
        let mut mod_path = paths::mod_path();
        mod_path = mod_path.join(&format!("{}.zip", &dir));

        let mut temp_path = paths::temp_path();
        temp_path = temp_path.join(&dir);

        zip_directory(&temp_path, &mod_path)?;
    }

    console::add_line(&app_handle, format!("<span class=\"console-green\">[Junimo] Installed {}</span>", &mod_name));
    &app_handle.emit_all("reload", false).unwrap();
    Ok(())
}

fn zip_directory(src_dir: &Path, dst_file: &Path) -> zip::result::ZipResult<()> {
    let file = File::create(dst_file)?;
    let walkdir = WalkDir::new(&src_dir);
    let it = walkdir.into_iter();

    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    let mut zip = ZipWriter::new(file);

    for entry in it.filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = path.strip_prefix(Path::new(&src_dir)).unwrap();

        // Check if it is a directory or a file
        if path.is_file() {
            zip.start_file(name.to_string_lossy(), options)?;
            let mut f = File::open(path)?;
            io::copy(&mut f, &mut zip)?;
        } else if name.as_os_str().len() != 0 {
            zip.add_directory(name.to_string_lossy(), options)?;
        }
    }
    zip.finish()?;
    Ok(())
}

pub fn extract_zip<R: io::Read + io::Seek>(mut archive: ZipArchive<R>, destination: &Path) -> zip::result::ZipResult<()> {
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => destination.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            if !outpath.exists() {
                fs::create_dir_all(&outpath)?;
            }
        } else {
            if let Some(parent) = outpath.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode))?;
            }
        }
    }
    Ok(())
}