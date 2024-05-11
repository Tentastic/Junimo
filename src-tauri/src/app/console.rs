use mockall::automock;
use serde::{Deserialize, Serialize};
use tauri::test::MockRuntime;
use tauri::{AppHandle, Manager, Runtime};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
struct Console {
    content: String,
    mode: i32,
}

pub fn empty_line<R: Runtime>(handle: &AppHandle<R>) {
    let console_content = Console {
        content: "⠀".to_string(),
        mode: 2,
    };
    &handle.emit("console", console_content).unwrap();
}

pub fn add_line<R: Runtime>(handle: &AppHandle<R>, content: String) {
    let console_content = Console {
        content: content.to_string(),
        mode: 0,
    };
    &handle.emit("console", console_content).unwrap();
}

pub fn modify_line<R: Runtime>(handle: &AppHandle<R>, content: String) {
    let console_content = Console {
        content: content.to_string(),
        mode: 1,
    };
    &handle.emit("console", console_content).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_line() {
        let tauri_app = tauri::test::mock_app();
        let handle = tauri_app.handle().clone();

        let listener = tauri_app.listen("console", |e| {
            let expected_payload = Console {
                content: "⠀".to_string(),
                mode: 2,
            };
            let serialized_payload = serde_json::to_string(&expected_payload).unwrap();
            assert_eq!(serialized_payload, e.payload());
        });

        empty_line(&handle);
    }

    #[test]
    fn test_add_line() {
        let tauri_app = tauri::test::mock_app();
        let handle = tauri_app.handle().clone();

        let listener = tauri_app.listen("console", |e| {
            let expected_payload = Console {
                content: "Hello World".to_string(),
                mode: 0,
            };
            let serialized_payload = serde_json::to_string(&expected_payload).unwrap();
            assert_eq!(serialized_payload, e.payload());
        });

        add_line(&handle, "Hello World".to_string());
    }

    #[test]
    fn test_modify_line() {
        let tauri_app = tauri::test::mock_app();
        let handle = tauri_app.handle().clone();

        let listener = tauri_app.listen("console", |e| {
            let expected_payload = Console {
                content: "Hello World".to_string(),
                mode: 1,
            };
            let serialized_payload = serde_json::to_string(&expected_payload).unwrap();
            assert_eq!(serialized_payload, e.payload());
        });

        modify_line(&handle, "Hello World".to_string());
    }
}
