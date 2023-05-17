use serde::Deserialize;
use serde::Serialize;

// Domain types
#[derive(Clone)]
pub struct Astronaut {
    pub id: String,
    pub name: String,
    pub password: String,
}

// Mongo types
#[derive(Clone, Deserialize, Serialize)]
pub struct AstronautDocument {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub password: String,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct AstronautUpdatedDocument {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

// Kafka types
#[derive(Clone, Deserialize, Serialize)]
pub struct AstronautCreatedEvent {
    pub id: String,
    pub name: String,
    pub password: String,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct AstronautUpdatedEvent {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

// Transformation between types
impl From<&AstronautDocument> for Astronaut {
    fn from(input: &AstronautDocument) -> Self {
        Self {
            id: input.id.clone(),
            name: input.name.clone(),
            password: input.password.clone(),
        }
    }
}

impl From<&AstronautCreatedEvent> for AstronautDocument {
    fn from(input: &AstronautCreatedEvent) -> Self {
        Self {
            id: input.id.clone(),
            name: input.name.clone(),
            password: input.password.clone(),
        }
    }
}

impl From<&AstronautUpdatedEvent> for AstronautUpdatedDocument {
    fn from(input: &AstronautUpdatedEvent) -> Self {
        Self {
            name: input.name.clone(),
            password: input.password.clone(),
        }
    }
}
