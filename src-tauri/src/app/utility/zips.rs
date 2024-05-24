use std::ffi::OsStr;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::{fs, io};

use tauri::{AppHandle, Manager};
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

use crate::app::profiles::Profile;
use crate::app::utility::paths;
use crate::app::{console, mod_installation, profiles};

/// Unpacks a new installed zip file
pub fn unpack_zip<R: io::Read + io::Seek>(
    app_handle: &AppHandle,
    mut archive: ZipArchive<R>,
    destination: &Path,
    max: usize,
) -> Result<String, String> {
    let mut main_dir = "".to_string();
    let mut has_manifest = false;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let replaced = file.name().replace("\\", "/");
        let split = replaced.split('/').collect::<Vec<&str>>();

        if split[0] != main_dir {
            main_dir = split[0].to_string();
        }

        if replaced.contains("__MACOSX") {
            continue;
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
                match std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode)) {
                    Ok(_) => {}
                    Err(e) => {
                        return Err(e.to_string());
                    }
                };
            }
        }

        console::modify_line(
            &app_handle,
            mod_installation::install_progress(&i, &max).to_string(),
        );
    }

    return if !has_manifest {
        Err("No manifest found".to_string())
    } else {
        Ok(main_dir)
    };
}

/// Unpacks a new installed zip file
pub fn unpack_smapi<R: io::Read + io::Seek>(
    mut archive: ZipArchive<R>,
    destination: &Path
) -> Result<String, String> {
    let mut main_dir = "".to_string();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();

        if main_dir == "" {
            let new_filepath = file.name().to_string().replace("\\", "/");
            let split = new_filepath.split('/').collect::<Vec<&str>>();
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
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                match std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode)) {
                    Ok(_) => {}
                    Err(e) => {
                        return Err(e.to_string());
                    }
                };
            }
        }
    }

    Ok(main_dir)
}

/// Writes mod files into a zip archive
///
/// * `zip` - The zip writer to write to
/// * `src_dir` - The source directory to zip
///
/// # Returns a zip result
pub fn zip_mods(zip: &mut ZipWriter<File>, src_dir: &PathBuf) -> zip::result::ZipResult<()> {
    let walkdir = WalkDir::new(src_dir);
    let it = walkdir.into_iter();

    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    let src_dir_name = src_dir.file_name().unwrap().to_string_lossy();

    for entry in it.filter_map(|e| e.ok()) {
        let path = entry.path();
        let mut zip_path = PathBuf::from("mods"); // Start with the source directory name
        if !&src_dir_name.to_string().contains(".") {
            zip_path = zip_path.join(format!(".{}", &src_dir_name.to_string()));
        } else {
            zip_path = zip_path.join(&src_dir_name.to_string());
        }
        zip_path = zip_path.join(path.strip_prefix(src_dir).unwrap());

        if path.is_file() {
            zip.start_file(zip_path.to_string_lossy(), options)?;
            let mut f = File::open(path)?;
            io::copy(&mut f, zip)?;
        } else if !zip_path.as_os_str().is_empty() {
            zip.add_directory(zip_path.to_string_lossy(), options)?;
        }
    }

    Ok(())
}

/// Import an exported zip file
///
/// * `archive` - The zip archive to import
/// * `destination` - The destination path to import to
/// * `temp_path` - The temporary path to extract the metafiles like profile.json or mods.json into
/// * `all` - Whether the import should overwrite all existing profiles
///
/// # Returns a zip result
pub fn import_zip<R: io::Read + io::Seek>(
    mut archive: ZipArchive<R>,
    destination: &Path,
    temp_path: &Path,
    all: bool,
) -> zip::result::ZipResult<()> {
    // Go through all files in zip file
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        match file.enclosed_name() {
            Some(path) => {
                if path.starts_with("mods") {
                    import_mod_file(destination, &path, &mut file, all)?;
                } else {
                    import_metadata_file(destination, temp_path, &path, &mut file, all)?;
                }
            }
            None => continue,
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                std::fs::set_permissions(&destination, std::fs::Permissions::from_mode(mode))?;
            }
        }
    }
    Ok(())
}

/// Import a mod file from a zip archive
///
/// * `destination` - The destination path to import to
/// * `path` - Path of the mod in the zip file
/// * `file` - The file to import
/// * `all` - Whether the import should overwrite all existing profiles
///
/// # Returns a zip result
fn import_mod_file(
    destination: &Path,
    path: &Path,
    file: &mut zip::read::ZipFile,
    all: bool,
) -> zip::result::ZipResult<()> {
    let mod_dest = destination.join(&path);
    let mut path_iter = path.iter();
    let dir_name = path_iter.nth(1).unwrap();
    let path_in_mods = paths::mod_path().join(dir_name);
    let path_in_mods_dot = paths::mod_path().join(format!(".{:?}", dir_name));

    if path_iter.clone().count() == 0 {
        if (path_in_mods.exists() || path_in_mods_dot.exists()) && !all {
            return Ok(());
        } else if (path_in_mods.exists() || path_in_mods_dot.exists()) && all {
            remove_if_exists(path_in_mods);
            remove_if_exists(path_in_mods_dot);
        }
    }

    if file.is_dir() {
        fs::create_dir_all(mod_dest).unwrap();
    } else {
        if let Some(parent) = mod_dest.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        let mut outfile = File::create(&mod_dest)?;
        io::copy(file, &mut outfile)?;
    }

    Ok(())
}

/// Remove a file or directory if it exists
///
/// * `path` - The path to remove
fn remove_if_exists(path: PathBuf) {
    if path.exists() && path.is_dir() {
        if path.is_dir() {
            fs::remove_dir_all(&path).unwrap();
        } else {
            fs::remove_file(&path).unwrap();
        }
    }
}

/// Import a metadata file from a zip archive (profile.json or mods.json)
///
/// * `destination` - The destination path to import to
/// * `temp_path` - The temporary path to extract the metafiles like profile.json or mods.json into
/// * `path` - Path of the metadata file in the zip file
/// * `file` - The file to import
/// * `all` - Whether the import should overwrite all existing profiles
///
/// # Returns a zip result
fn import_metadata_file(
    destination: &Path,
    temp_path: &Path,
    path: &Path,
    file: &mut zip::read::ZipFile,
    all: bool,
) -> zip::result::ZipResult<()> {
    let outpath = temp_path.join(path);

    if file.name().contains("json") {
        let mut outfile = File::create(&outpath)?;
        io::copy(file, &mut outfile)?;

        if all {
            let profile_dest = destination.join("profile.json");
            let mods_dest = destination.join("mods.json");

            if file.name().contains("profile.json") {
                if profile_dest.exists() {
                    fs::remove_file(&profile_dest).unwrap();
                }

                fs::rename(&outpath, &profile_dest).unwrap();
            } else if file.name().contains("mods.json") {
                if mods_dest.exists() {
                    fs::remove_file(&mods_dest).unwrap();
                }

                fs::rename(&outpath, &mods_dest).unwrap();
            }
        } else {
            let data_raw = fs::read_to_string(&outpath).unwrap();
            let data = data_raw.as_str();
            let loaded_profiles: Vec<Profile> = serde_json::from_str(data).unwrap();

            if loaded_profiles.len() == 0 {
                return Ok(());
            }

            let loaded_profile = loaded_profiles[0].clone();
            let current_profiles = profiles::get_profiles(paths::profile_path());

            let mut current_profiles = current_profiles
                .into_iter()
                .filter(|prof| prof.name != loaded_profile.name)
                .collect::<Vec<Profile>>();
            current_profiles.push(loaded_profile);

            profiles::save_profiles(&current_profiles, &paths::profile_path());
        }
    }

    Ok(())
}
