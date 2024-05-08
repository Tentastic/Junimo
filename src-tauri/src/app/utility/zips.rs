use std::{fs, io};
use std::fs::File;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};
use walkdir::WalkDir;
use zip::write::{FileOptions, SimpleFileOptions};
use zip::{CompressionMethod, ZipArchive, ZipWriter};
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

pub fn zip_directory(src_dir: &Path, dst_file: &Path)-> zip::result::ZipResult<()> {
    let file = File::create(dst_file)?;
    let walkdir = WalkDir::new(src_dir);
    let it = walkdir.into_iter();

    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    let mut zip = ZipWriter::new(file);
    let src_dir_name = src_dir.file_name().unwrap().to_string_lossy();

    for entry in it.filter_map(|e| e.ok()) {
        let path = entry.path();
        let mut zip_path = PathBuf::from(&src_dir_name.to_string());  // Start with the source directory name
        zip_path.push(path.strip_prefix(src_dir).unwrap());

        // Check if it is a directory or a file
        if path.is_file() {
            zip.start_file(zip_path.to_string_lossy(), options)?;
            let mut f = File::open(path)?;
            io::copy(&mut f, &mut zip)?;
        } else if !zip_path.as_os_str().is_empty() {
            // Add directories except the root
            zip.add_directory(zip_path.to_string_lossy(), options)?;
        }
    }
    zip.finish()?;
    Ok(())
}

pub fn extract_zip<R: io::Read + io::Seek>(mut archive: ZipArchive<R>, destination: &Path) -> zip::result::ZipResult<()> {
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let replaced_string = file.name().replace("\\", "/");
        let split = replaced_string.split('/').collect::<Vec<&str>>();
        if split.len() < 3 {
            if split[1].is_empty() {
                continue;
            }
        }


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