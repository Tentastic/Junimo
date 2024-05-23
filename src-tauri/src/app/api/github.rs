use crate::app::console;
use crate::app::utility::version_extractor;
use tauri::{AppHandle, command};

#[command]
pub async fn check_smapi_version() -> Vec<String> {
    let client = reqwest::Client::new();
    let res = client
        .get("https://api.github.com/repos/Pathoschild/SMAPI/releases/latest")
        .header("accept", "application/json")
        .header("User-Agent", "Junimo Client")
        .send()
        .await;

    #[cfg(target_family = "windows")]
    let smapi_version = version_extractor::get_version("StardewModdingAPI.exe");
    #[cfg(target_family = "unix")]
    let smapi_version = version_extractor::get_version("StardewModdingAPI.dll");

    return match res {
        Ok(res) => {
            let body = res.text().await.unwrap();
            let json: serde_json::Value = serde_json::from_str(&body).unwrap();
            let version = json["tag_name"].as_str().unwrap();
            let download_url = json["assets"][0]["browser_download_url"].as_str().unwrap();
            let changelog = json["body"].as_str().unwrap();
            let version = version.to_string();
            let download_url = download_url.to_string();

            if smapi_version.is_none() {
                println!("None");
                let download_url = download_url.replace("-for-developers", "");
                vec![download_url, version]
            }
            else if version == smapi_version.clone().unwrap() {
                let download_url = download_url.replace("-for-developers", "");
                vec![download_url, version, smapi_version.unwrap().to_string()]
            }
            else {
                vec![]
            }
        }
        Err(e) => {
            println!("{}", e);
            vec![]
        }
    }
}
