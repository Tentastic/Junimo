use std::collections::HashMap;

use crate::app::models::mod_info::ModInfo;
use reqwest::Response;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::app::utility::version_extractor;

/// Struct to extract id and installed version of each mod
#[derive(Serialize, Deserialize, Debug, Clone)]
struct SmapiMod {
    #[serde(rename = "id")]
    id: String,
    #[serde(rename = "installedVersion")]
    installed_version: String,
}

/// Wraps mods into a wrapper for the post request
#[derive(Serialize, Deserialize, Debug, Clone)]
struct SmapiPostWrapper {
    #[serde(rename = "mods")]
    mods: Vec<SmapiMod>,
    #[serde(rename = "apiVersion")]
    api_version: Option<String>,
    #[serde(rename = "gameVersion")]
    game_version: String,
    platform: String,
    #[serde(rename = "includeExtendedMetadata")]
    include_extend_metadata: bool,
}

impl SmapiPostWrapper {
    pub fn new(mods: Vec<SmapiMod>, api_version: Option<String>, game_version: String) -> Self {
        SmapiPostWrapper {
            mods,
            api_version,
            game_version,
            #[cfg(target_os = "windows")]
            platform: "Windows".to_string(),
            #[cfg(target_os = "linux")]
            platform: "Linux".to_string(),
            #[cfg(target_os = "macos")]
            platform: "Mac".to_string(),
            include_extend_metadata: true,
        }
    }
}

/// Struct to extract the result of the post request
#[derive(Serialize, Deserialize, Debug, Clone)]
struct SmapiResult {
    mods: Vec<SmapiWrapper>,
}

/// Struct that wraps the data of each mod in the post request
#[derive(Serialize, Deserialize, Debug, Clone)]
struct SmapiWrapper {
    id: String,
    #[serde(rename = "suggestedUpdate")]
    suggested_update: Option<SuggestesUpdate>,
    metadata: SmapiMeta,
    errors: Vec<String>,
}

/// Struct that wraps the suggested update of each mod
#[derive(Serialize, Deserialize, Debug, Clone)]
struct SuggestesUpdate {
    version: String,
    url: String,
}

/// Struct that contains all metadata of a smapi mod
#[derive(Serialize, Deserialize, Debug, Clone)]
struct SmapiMeta {
    id: Vec<String>,
    name: Option<String>,
    #[serde(rename = "nexusID")]
    nexus_id: Option<i32>,
    #[serde(rename = "gitHubRepo")]
    github_repo: Option<String>,
    #[serde(rename = "customSourceUrl")]
    custom_source_url: Option<String>,
    main: Option<SmapiMain>,
    optional: Option<SmapiMain>,
    unofficial: Option<SmapiMain>,
    #[serde(rename = "unofficialForBeta")]
    unofficial_for_beta: Option<String>,
    #[serde(rename = "hasBetaInfo")]
    has_beta_info: Option<String>,
    #[serde(rename = "compatibilityStatus")]
    compability_status: Option<String>,
    #[serde(rename = "compatibilitySummary")]
    compability_summary: Option<String>,
    #[serde(rename = "brokeIn")]
    broke_in: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SmapiMain {
    version: String,
    url: String,
}

/// Get the compatibility of a list of mods
///
/// * `mods` - The list of mods to check compatibility for
///
/// # Returns the same list of mods with the updated compatibility info
pub async fn get_compability(mods: Vec<ModInfo>) -> Option<Vec<ModInfo>> {
    let url = Url::parse("https://smapi.io/api/v3.0/mods").unwrap();

    // Get game and api version
    let mut game_version = "".to_string();
    match version_extractor::get_version("Stardew Valley.exe") {
        Some(version) => {
            game_version = version;
        }
        None => {}
    }
    let api_version: Option<String> = version_extractor::get_version("StardewModdingAPI.dll");

    // Collect mods to post by putting them into a vector of SmapiMod's
    let mut mods_post: Vec<SmapiMod> = vec![];
    for mod_info in mods.clone() {
        match mod_info.unique_id {
            Some(unique) => {
                let smapi_mod = SmapiMod {
                    id: unique.clone(),
                    installed_version: mod_info.version,
                };
                mods_post.push(smapi_mod);
            }
            None => {}
        }
    }

    // Create the post body
    let post_body_wrapper = SmapiPostWrapper::new(mods_post, api_version, game_version);

    // Send the post request
    let client = reqwest::Client::new();
    let res = client
        .post(url)
        .body(serde_json::to_string(&post_body_wrapper).unwrap())
        .header("Application-Name", "application/json")
        .header("User-Agent", "Junimo")
        .header("Content-Type", "application/json")
        .header("accept", "application/json")
        .send()
        .await
        .unwrap();

    return if res.status().is_success() {
        process_post_response(res, mods).await
    } else {
        println!("Error: {:?}", res.status());
        None
    };
}

/// Process the response of the post request
///
/// * `res` - The response of the post request
/// * `mods` - The list of mods to update
///
/// # Returns the list of mods with the updated compatibility info
async fn process_post_response(res: Response, mods: Vec<ModInfo>) -> Option<Vec<ModInfo>> {
    return match res.text().await {
        Ok(text) => {
            // Deserialize the response into a SmapiResult
            let result: Result<Vec<SmapiWrapper>, serde_json::Error> =
                serde_json::from_str(text.as_str());
            match result {
                Ok(wrapper) => {
                    let mut new_mods: Vec<ModInfo> = mods.clone();

                    // Create a hashmap to get the index of a mod by its unique id
                    let hash_map: HashMap<_, _> = mods
                        .clone()
                        .into_iter()
                        .enumerate()
                        .map(|(index, modinfo)| (modinfo.unique_id, index))
                        .collect();

                    // Iterate over the wrapper and update the mods with the compatibility info
                    for mod_info in wrapper {
                        if hash_map.contains_key(&Some(mod_info.id.clone())) {
                            let get_index = hash_map.get(&Some(mod_info.id.clone())).unwrap();
                            let mut copied_mod = mods[*get_index].clone();

                            match mod_info.metadata.compability_status {
                                Some(status) => {
                                    if status != "Ok" {
                                        if &mod_info
                                            .metadata
                                            .compability_summary
                                            .clone()
                                            .unwrap()
                                            .contains(copied_mod.version.as_str())
                                            == &false
                                        {
                                            let more_info = format!(
                                                "<span class=\"console-{}\">{}</span>",
                                                status.to_lowercase(),
                                                mod_info
                                                    .metadata
                                                    .compability_summary
                                                    .unwrap()
                                                    .replace("<a", "<a target=\"_blank\"")
                                            );
                                            copied_mod.more_info = Some(more_info);
                                            copied_mod.is_broken = Some(true);
                                            new_mods[*get_index] = copied_mod.clone();
                                        } else {
                                            copied_mod.more_info = None;
                                            copied_mod.is_broken = None;
                                            new_mods[*get_index] = copied_mod.clone();
                                        }
                                    }
                                }
                                None => {
                                    new_mods[*get_index] = copied_mod.clone();
                                }
                            }
                            new_mods[*get_index] = copied_mod;
                        }
                    }

                    Some(new_mods)
                }
                Err(e) => None,
            }
        }
        Err(e) => None,
    };
}
