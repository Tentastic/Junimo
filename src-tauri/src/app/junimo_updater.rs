use tauri::{command, WebviewUrl};

#[command]
pub async fn open_updater(handle: tauri::AppHandle) {
    #[cfg(target_os = "windows")]
    tauri::WebviewWindowBuilder::new(&handle, "Updater", WebviewUrl::App("/updater".into()))
        .title("Updater")
        .resizable(false)
        .maximizable(false)
        .inner_size(700.0, 350.0)
        .transparent(true)
        .build()
        .unwrap();

    #[cfg(target_os = "unix")]
    tauri::WebviewWindowBuilder::new(&handle, "Updater", WebviewUrl::App("/updater".into()))
        .title("Updater")
        .resizable(false)
        .maximizable(false)
        .inner_size(600.0, 350.0)
        .build()
        .unwrap();
}