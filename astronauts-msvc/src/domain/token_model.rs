use serde::Deserialize;
use serde::Serialize;

// Pure types for domain use
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccessTokenPayload {
    #[serde(rename = "fid")]
    pub family_id: String,
    #[serde(rename = "aid")]
    pub astronaut_id: String,
    #[serde(rename = "per")]
    pub permissions: Vec<Permission>,
}

// Permissions enum
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Permission {
    GetAnyAstronaut,
    GetOwnAstronaut,
    UpdateAnyAstronaut,
    UpdateOwnAstronaut,
}
