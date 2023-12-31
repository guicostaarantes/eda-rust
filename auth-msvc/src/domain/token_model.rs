use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

pub const PASSWORD_GRANT_PERMISSIONS: [Permission; 5] = [
    Permission::GetAstronaut,
    Permission::UpdateAstronaut,
    Permission::CreateMission,
    Permission::UpdateMission,
    Permission::GetMission,
];
pub const REFRESH_TOKEN_EXPIRES_IN_SECONDS: u64 = 604800;
pub const ACCESS_TOKEN_EXPIRES_IN_SECONDS: u64 = 900;

// Domain types
#[derive(Serialize, Deserialize)]
pub struct TokenPair {
    pub refresh_token: String,
    pub access_token: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RefreshTokenPayload {
    #[serde(rename = "fid")]
    pub family_id: String,
    #[serde(rename = "aid")]
    pub astronaut_id: String,
    #[serde(rename = "atp")]
    pub permissions: Vec<Permission>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccessTokenPayload {
    #[serde(rename = "fid")]
    pub family_id: String,
    #[serde(rename = "aid")]
    pub astronaut_id: String,
    #[serde(rename = "per")]
    pub permissions: Vec<Permission>,
}

// input types
#[derive(Serialize, Deserialize)]
pub struct AstronautCredentialsInput {
    pub name: String,
    pub password: String,
}

// Mongo types
#[derive(Clone, Deserialize, Serialize)]
pub struct RefreshTokenDocument {
    #[serde(rename = "_id")]
    pub id: String,
    pub signature: String,
    pub astronaut_id: String,
    pub permissions: Vec<Permission>,
    pub expires_at: DateTime<Utc>,
}

// Kafka types
#[derive(Clone, Deserialize, Serialize)]
pub struct RefreshTokenCreatedEvent {
    pub id: String,
    pub signature: String,
    pub astronaut_id: String,
    pub permissions: Vec<Permission>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct RefreshTokenRevokedEvent {
    pub id: String,
}

// Transformation between types
impl From<&RefreshTokenCreatedEvent> for RefreshTokenDocument {
    fn from(input: &RefreshTokenCreatedEvent) -> Self {
        Self {
            id: input.id.clone(),
            signature: input.signature.clone(),
            astronaut_id: input.astronaut_id.clone(),
            permissions: input.permissions.clone(),
            expires_at: Utc::now()
                + chrono::Duration::seconds(REFRESH_TOKEN_EXPIRES_IN_SECONDS as i64),
        }
    }
}

// Permissions enum
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Permission {
    GetAstronaut,
    UpdateAstronaut,
    CreateMission,
    UpdateMission,
    GetMission,
}
