use rfd::FileDialog;


pub trait DirectorySelector: Send + Sync {
    fn pick_directory(&self, initial_path: &str) -> Option<std::path::PathBuf>;
}

#[derive(Clone)]
pub struct RealDirectorySelector;
impl DirectorySelector for RealDirectorySelector {
    fn pick_directory(&self, initial_path: &str) -> Option<std::path::PathBuf> {
        FileDialog::new().set_directory(initial_path).pick_folder()
    }
}

#[derive(Clone)]
pub struct MockDirectorySelector;
impl DirectorySelector for MockDirectorySelector {
    fn pick_directory(&self, initial_path: &str) -> Option<std::path::PathBuf> {
        if initial_path == " " {
            return None;
        }
        Some(std::path::PathBuf::from(initial_path))
    }
}