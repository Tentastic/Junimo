use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Console {
    content: String,
    mode: i32
}

pub fn empty_line(handle: &AppHandle) {
    let console_content = Console {
        content: "⠀".to_string(),
        mode: 2
    };
    &handle.emit_all("console", console_content).unwrap();
}

pub fn add_line(handle: &AppHandle, content: String) {
    let console_content = Console {
        content: content.to_string(),
        mode: 0
    };
    &handle.emit_all("console", console_content).unwrap();
}

pub fn modify_line(handle: &AppHandle, content: String) {
    let console_content = Console {
        content: content.to_string(),
        mode: 1
    };
    &handle.emit_all("console", console_content).unwrap();
}