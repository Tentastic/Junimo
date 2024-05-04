use std::fs::File;
use std::io::{Read, Write};
use serde::{Deserialize, Serialize};
use tauri::command;
use futures_util::{SinkExt};
use bincode;
use app::api::nexuswebsocket;
use crate::app;
use crate::app::util::app_path;

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    user_id: i64,
    key: String,
    name: String,
    is_premium: Option<bool>,
    is_supporter: Option<bool>,
    profile_url: String
}

#[command]
pub fn load_user() -> Option<User> {
    if app_path("user.stp").exists() {
        let mut file = File::open(app_path("user.stp")).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        let user : User = bincode::deserialize(&buffer[..]).unwrap();

        return Some(user)
    }

    None
}

fn save_user(user: &User) {
    let encoded: Vec<u8> = bincode::serialize(&user).unwrap();
    let mut file = File::create(app_path("user.stp")).unwrap();
    file.write_all(&encoded).unwrap();
}

#[command]
pub async fn connect_user(handle: tauri::AppHandle) -> Option<User> {
    nexuswebsocket::connect_user(handle).await;
    user_info().await
}

async fn user_info() -> Option<User> {
    let key = nexuswebsocket::load_key();

    let loaded_user = load_user();

    match loaded_user {
        None => {
            let client = reqwest::Client::new();
            let res = client.get("https://api.nexusmods.com/v1/users/validate.json")
                .header("accept", "application/json")
                .header("apikey", key)
                .send()
                .await
                .unwrap();

            if res.status().is_success() {
                let body = res.text().await.unwrap();
                let body_str = body.as_str();
                let user: User = serde_json::from_str(body_str).unwrap();
                save_user(&user);
                Some(user)
            } else {
                None
            }
        },
        Some(value) => {
            Some(value)
        },
    }
}

