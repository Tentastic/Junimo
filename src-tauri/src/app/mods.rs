use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use rfd::FileDialog;
use zip::read::ZipArchive;
use std::{env, fs};

use tauri::{command};

#[command]
pub fn add_mod() {
    let file = FileDialog::new()
        .add_filter("zip", &["zip"])
        .add_filter("rar", &["rar"])
        .set_directory("/")
        .pick_file();

    if let Some(path) = file {
        fs::create_dir_all("/mods");

        let zip_file_path = path;

        let mut output_folder_path = env::current_exe()
            .expect("Failed to get the executable path");
        output_folder_path.pop();
        output_folder_path.push("mods");


        // Opening the ZIP file
        let zip_file = File::open(&zip_file_path).unwrap();
        let zip_archive = ZipArchive::new(zip_file).unwrap();

        // Extracting the ZIP file
        extract_zip(zip_archive, &output_folder_path);
    } else {
        println!("No file was selected.");
    }
}

fn extract_zip<R: io::Read + io::Seek>(mut archive: ZipArchive<R>, destination: &Path) -> zip::result::ZipResult<()> {
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => destination.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            if !outpath.exists() {
                std::fs::create_dir_all(&outpath)?;
            }
        } else {
            if let Some(parent) = outpath.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }

        // Set the permissions if on Unix
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