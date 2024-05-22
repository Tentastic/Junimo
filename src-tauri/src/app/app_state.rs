use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct AppState {
    pub stop_download: Arc<Mutex<bool>>,
    pub stop_game: Arc<Mutex<bool>>,
}

impl AppState {
    pub fn new() -> (Self, mpsc::Receiver<String>) {
        let (tx, rx) = mpsc::channel(32);
        (
            AppState {
                stop_download: Arc::new(Mutex::new(false)),
                stop_game: Arc::new(Mutex::new(true)),
            },
            rx,
        )
    }
}
