use crate::app::mods::Dependency;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModInfo {
    pub name: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub picture_url: Option<String>,
    pub mod_downloads: u64,
    pub mod_unique_downloads: u64,
    pub uid: u64,
    pub mod_id: u32,
    pub game_id: u32,
    pub allow_rating: bool,
    pub domain_name: String,
    pub category_id: u32,
    pub version: String,
    pub endorsement_count: u32,
    pub created_timestamp: u64,
    pub created_time: String,
    pub updated_timestamp: u64,
    pub updated_time: String,
    pub author: String,
    pub uploaded_by: String,
    pub uploaded_users_profile_url: String,
    pub contains_adult_content: bool,
    pub status: String,
    pub available: bool,
    pub unique_id: Option<String>,
    pub more_info: Option<String>,
    pub dependencies: Option<Vec<Dependency>>,
    pub group: Option<String>,
    pub is_broken: Option<bool>,
}

impl PartialEq for ModInfo {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Hash for ModInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state); // Only hash the name
    }
}
