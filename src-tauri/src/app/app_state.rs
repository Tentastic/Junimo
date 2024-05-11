use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use crate::testing::directory_selector::DirectorySelector;

#[derive(Clone)]
pub struct AppState {
    pub stop_download: Arc<Mutex<bool>>,
    pub stop_game: Arc<Mutex<bool>>,
    pub directory_selector: Arc<Mutex<Box<dyn DirectorySelector>>>,
}

impl AppState {
    pub fn new(selector: Box<dyn DirectorySelector>) -> (Self, mpsc::Receiver<String>) {
        let (tx, rx) = mpsc::channel(32);
        (
            AppState {
                stop_download: Arc::new(Mutex::new(false)),
                stop_game: Arc::new(Mutex::new(true)),
                directory_selector: Arc::new(Mutex::new(selector))
            },
            rx,
        )
    }
}
