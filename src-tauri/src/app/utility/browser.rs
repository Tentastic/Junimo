#[cfg(target_os = "windows")]
pub fn open_url(url: &str) {
    std::process::Command::new("cmd")
        .args(&["/C", "start", url])
        .spawn()
        .expect("Failed to open URL with default browser");
}

#[cfg(target_os = "linux")]
pub fn open_url(url: &str) {
    std::process::Command::new("xdg-open")
        .arg(url)
        .spawn()
        .expect("Failed to open URL with default browser");
}

#[cfg(target_os = "macos")]
pub fn open_url(url: &str) {
    std::process::Command::new("open")
        .arg(url)
        .spawn()
        .expect("Failed to open URL with default browser");
}
