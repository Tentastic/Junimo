use url::Url;
use crate::app::api::nexuswebsocket;
use crate::app::mods::ModInfo;

pub fn get_url(url_str: &str) -> String{
    let url = Url::parse(url_str).unwrap();
    let path_segments: Vec<&str> = url.path_segments().map(|c| c.collect()).unwrap_or_else(Vec::new);
    let mut query_pairs = url.query_pairs();

    if query_pairs.count() == 0 {
        let api_url = format!(
            "https://api.nexusmods.com/v1/games/{}/mods/{}/files/{}/download_link.json",
            url.domain().unwrap(),
            path_segments[1],
            path_segments[3]
        );
        api_url
    }
    else {
        let (key1, value1) = query_pairs.next().unwrap();
        let (key2, value2) = query_pairs.next().unwrap();

        let api_url = format!(
            "https://api.nexusmods.com/v1/games/{}/mods/{}/files/{}/download_link.json?key={}&expires={}",
            url.domain().unwrap(),
            path_segments[1],
            path_segments[3],
            value1,
            value2
        );
        api_url
    }
}

pub async fn get_infos(url_str: &str) -> Option<ModInfo> {
    let url = Url::parse(url_str).unwrap();
    let path_segments: Vec<&str> = url.path_segments().map(|c| c.collect()).unwrap_or_else(Vec::new);
    let api_url = format!(
        "https://api.nexusmods.com/v1/games/{}/mods/{}.json",
        url.domain().unwrap(),
        path_segments[1]
    );

    let client = reqwest::Client::new();
    let res = client.get(&api_url)
        .header("accept", "application/json")
        .header("apikey", nexuswebsocket::load_key())
        .send()
        .await
        .unwrap();

    println!("Scheme: {}", &api_url);

    if res.status().is_success() {
        let body = res.text().await.unwrap();
        let body_str = body.as_str();
        let mod_info: ModInfo = serde_json::from_str(body_str).unwrap();
        Some(mod_info)
    }
    else {
        None
    }
}