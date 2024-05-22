use std::ffi::OsString;
use widestring::U16CString;
use winapi::shared::minwindef::{DWORD, LPVOID};
use std::fs;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;
use winapi::um::winver::{GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW};
use crate::app::utility::{paths, version_extractor};

/// Struct that forms a dll version
#[derive(Debug)]
struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
    build: u16,
}

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