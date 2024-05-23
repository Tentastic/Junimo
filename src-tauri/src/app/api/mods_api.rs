use url::form_urlencoded::Parse;
use url::Url;

use crate::app::api::nexuswebsocket;
use crate::app::models::mod_info::ModInfo;

pub fn get_download_link(url_str: &str) -> String {
    let url = Url::parse(url_str).unwrap();
    let path_segments: Vec<&str> = url
        .path_segments()
        .map(|c| c.collect())
        .unwrap_or_else(Vec::new);
    let query_pairs = url.query_pairs();
    let domain = url.domain().unwrap().to_string();

    if query_pairs.count() < 2 {
        premium_link(domain.to_string(), path_segments)
    } else {
        link_request(domain.to_string(), path_segments, query_pairs)
    }
}

fn premium_link(domain: String, path_segments: Vec<&str>) -> String {
    let api_url = format!(
        "https://api.nexusmods.com/v1/games/{}/mods/{}/files/{}/download_link.json",
        domain, path_segments[1], path_segments[3]
    );
    api_url
}

fn link_request(domain: String, path_segments: Vec<&str>, mut query_pairs: Parse) -> String {
    let (_key1, value1) = query_pairs.next().unwrap();
    let (_key2, value2) = query_pairs.next().unwrap();

    let api_url = format!(
        "https://api.nexusmods.com/v1/games/{}/mods/{}/files/{}/download_link.json?key={}&expires={}",
        domain,
        path_segments[1],
        path_segments[3],
        value1,
        value2
    );
    api_url
}

pub async fn get_infos(url_str: &str) -> Option<ModInfo> {
    let url = Url::parse(url_str).unwrap();
    let path_segments: Vec<&str> = url
        .path_segments()
        .map(|c| c.collect())
        .unwrap_or_else(Vec::new);
    let api_url = format!(
        "https://api.nexusmods.com/v1/games/{}/mods/{}.json",
        url.domain().unwrap(),
        path_segments[1]
    );

    let client = reqwest::Client::new();
    let res = client
        .get(&api_url)
        .header("accept", "application/json")
        .header("apikey", nexuswebsocket::load_key())
        .send()
        .await
        .unwrap();

    if res.status().is_success() {
        let body = res.text().await.unwrap();
        let body_str = body.as_str();
        let mod_info: ModInfo = serde_json::from_str(body_str).unwrap();
        Some(mod_info)
    } else {
        None
    }
}

fn api_url(url_str: &str) -> String {
    let url = Url::parse(url_str).unwrap();
    let path_segments: Vec<&str> = url
        .path_segments()
        .map(|c| c.collect())
        .unwrap_or_else(Vec::new);
    let api_url = format!(
        "https://api.nexusmods.com/v1/games/{}/mods/{}.json",
        url.domain().unwrap(),
        path_segments[1]
    );
    api_url
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_download_link() {
        let url_str = "nxm://stardewvalley/mods/1915/files/92455?key=7PKaqYlhW6z-RNUOLSq3uQ&expires=1715679746&user_id=66607686";
        let result = get_download_link(url_str);
        assert_eq!(result, "https://api.nexusmods.com/v1/games/stardewvalley/mods/1915/files/92455/download_link.json?key=7PKaqYlhW6z-RNUOLSq3uQ&expires=1715679746");
    }

    #[test]
    fn test_get_download_link_premium() {
        let url_str = "nxm://stardewvalley/mods/1915/files/92455";
        let result = get_download_link(url_str);
        assert_eq!(result, "https://api.nexusmods.com/v1/games/stardewvalley/mods/1915/files/92455/download_link.json");
    }

    #[test]
    fn test_link_request() {
        let url_str = "nxm://stardewvalley/mods/1915/files/92455?key=7PKaqYlhW6z-RNUOLSq3uQ&expires=1715679746&user_id=66607686";
        let url = Url::parse(url_str).unwrap();
        let path_segments: Vec<&str> = url
            .path_segments()
            .map(|c| c.collect())
            .unwrap_or_else(Vec::new);
        let query_pairs = url.query_pairs();
        let domain = url.domain().unwrap().to_string();
        let link_request = link_request(
            domain.to_string().clone(),
            path_segments.clone(),
            query_pairs.clone(),
        );

        assert_eq!(path_segments, vec!["mods", "1915", "files", "92455"]);
        assert_eq!(query_pairs.count(), 3);
        assert_eq!(domain, "stardewvalley");
        assert_eq!(link_request, "https://api.nexusmods.com/v1/games/stardewvalley/mods/1915/files/92455/download_link.json?key=7PKaqYlhW6z-RNUOLSq3uQ&expires=1715679746");
    }

    #[tokio::test]
    async fn test_get_infos_some() {
        let url_str = "nxm://stardewvalley/mods/1915/files/92455";
        let result = get_infos(url_str).await;
        assert_eq!(result.is_none(), true);
    }

    #[tokio::test]
    async fn test_get_infos_none() {
        let url_str = "nxm://stardewvalley/mods/3/files/3";
        let result = get_infos(url_str).await;
        assert_eq!(result.is_none(), true);
    }

    #[test]
    fn test_api_url() {
        let url_str = "nxm://stardewvalley/mods/1915/files/92455?key=7PKaqYlhW6z-RNUOLSq3uQ&expires=1715679746&user_id=66607686";
        let result = api_url(url_str);
        assert_eq!(
            result,
            "https://api.nexusmods.com/v1/games/stardewvalley/mods/1915.json"
        );
    }
}
