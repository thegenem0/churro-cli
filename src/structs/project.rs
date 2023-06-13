use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectResponse {
    #[serde(rename = "self")]
    pub self_link: String,
    pub maxResults: u32,
    pub startAt: u32,
    pub total: u32,
    pub isLast: bool,
    pub values: Vec<Project>,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub expand: String,

    #[serde(rename = "self")]
    pub self_link: String,

    pub id: String,
    pub key: String,
    pub name: String,

    pub projectTypeKey: String,

    pub simplified: bool,
    pub style: String,

    pub isPrivate: bool,
    pub properties: HashMap<String, serde_json::Value>, // can be any valid JSON structure
    pub entityId: String,
    pub uuid: String,
}
