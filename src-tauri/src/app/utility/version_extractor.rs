use std::ffi::OsString;
use std::fs::File;
use std::io::Read;
use widestring::U16CString;
#[cfg(target_os = "windows")]
use winapi::shared::minwindef::{DWORD, LPVOID};
#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStrExt;
#[cfg(target_os = "windows")]
use winapi::um::winver::{GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW};
use crate::app::utility::{paths, version_extractor};

#[cfg(target_family = "unix")]
use pelite::pe64::{Pe, PeFile};
#[cfg(target_family = "unix")]
use pelite::resources::{FindError, Resources};

/// Struct that forms a dll version
#[derive(Debug)]
struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
    build: u16,
}

#[cfg(target_os = "windows")]
/// Struct to extract the version information from a DLL
#[repr(C)]
struct VsFixedfileinfo {
    dw_signature: DWORD,
    dw_struc_version: DWORD,
    dw_file_version_ms: DWORD,
    dw_file_version_ls: DWORD,
    dw_product_version_ms: DWORD,
    dw_product_version_ls: DWORD,
    dw_file_flags_mask: DWORD,
    dw_file_flags: DWORD,
    dw_file_os: DWORD,
    dw_file_type: DWORD,
    dw_file_subtype: DWORD,
    dw_file_date_ms: DWORD,
    dw_file_date_ls: DWORD,
}

#[cfg(target_os = "windows")]
/// Extracts the version of a dll file in the game directory
///
/// * `dll` - The name of the dll file
///
/// # Returns Version of the dll or none if the version could not be extracted
pub fn get_version(dll: &str) -> Option<String> {
    let path = paths::get_game_path().join(dll).to_string_lossy().to_string();
    return match get_version_info_from_dll(path.as_str()) {
        Some(version) => Some(format!("{}.{}.{}", version.major, version.minor, version.patch)),
        None => None,
    }
}

#[cfg(target_os = "windows")]
/// Extracts the version of our games and smapis dll
fn get_version_info_from_dll(path: &str) -> Option<Version> {
    // Convert path to a wide string
    let wide_path: Vec<u16> = OsString::from(path).encode_wide().chain(std::iter::once(0)).collect();

    // Get the size of the version info
    let mut handle: DWORD = 0;
    let size = unsafe { GetFileVersionInfoSizeW(wide_path.as_ptr(), &mut handle) };
    if size == 0 {
        return None;
    }

    // Read the version info into a buffer
    let mut buffer: Vec<u8> = vec![0; size as usize];
    let result = unsafe { GetFileVersionInfoW(wide_path.as_ptr(), handle as usize as DWORD, size, buffer.as_mut_ptr() as *mut _) };
    if result == 0 {
        return None;
    }

    // Extract the version information
    let mut lp_buffer: LPVOID = std::ptr::null_mut();
    let mut len: u32 = 0;
    let sub_block = U16CString::from_str("\\").unwrap();
    let result = unsafe { VerQueryValueW(buffer.as_ptr() as *const _, sub_block.as_ptr(), &mut lp_buffer, &mut len) };
    if result == 0 {
        return None;
    }

    // Safely dereference the pointer without moving the content
    unsafe {
        let version_info: &VsFixedfileinfo = &*(lp_buffer as *const VsFixedfileinfo);
        if version_info.dw_signature != 0xfeef04bd {
            return None;
        }

        Some(Version {
            major: (version_info.dw_file_version_ms >> 16) as u16,
            minor: (version_info.dw_file_version_ms & 0xFFFF) as u16,
            patch: (version_info.dw_file_version_ls >> 16) as u16,
            build: (version_info.dw_file_version_ls & 0xFFFF) as u16,
        })
    }
}

#[cfg(target_family = "unix")]
/// Extracts the version of a dll file in the game directory
///
/// * `dll` - The name of the dll file
///
/// # Returns Version of the dll or none if the version could not be extracted
pub fn get_version(dll: &str) -> Option<String> {
    let path = paths::get_game_path().join(dll).to_string_lossy().to_string();
    return match get_version_info_from_dll(path.as_str()) {
        Ok(version) => {
            match version {
                Some(version) => Some(format!("{}.{}.{}", version.major, version.minor, version.patch)),
                None => None,
            }
        },
        _ => None,
    }
}

#[cfg(target_family = "unix")]
/// Extracts the version of our games and smapis dll
fn get_version_info_from_dll(path: &str) -> Result<Option<Version>, String> {
    // Load the DLL file
    let file = File::open(path);
    if file.is_err() {
        return Err(file.err().unwrap().to_string());
    }
    let mut file = file.unwrap();

    let mut buffer = Vec::new();
    let _result = file.read_to_end(&mut buffer);
    if _result.is_err() {
        return Err(_result.err().unwrap().to_string());
    }

    // Parse the PE file
    let pe = PeFile::from_bytes(&buffer).expect("Failed to parse PE file");

    // Access the resources section
    let resources = pe.resources().expect("Failed to get resources");



    let version_info = resources.version_info();
    if version_info.is_err() {
        return Err(version_info.err().unwrap().to_string());
    }
    let version_info = version_info.unwrap();


    let fixed = version_info.fixed();
    if fixed.is_none() {
        return Ok(None);
    }
    let fixed = fixed.unwrap();

    let version = Version {
        major: fixed.dwFileVersion.Major,
        minor: fixed.dwFileVersion.Minor,
        patch: fixed.dwFileVersion.Patch,
        build: fixed.dwFileVersion.Build
    };

    Ok(Some(version))
}