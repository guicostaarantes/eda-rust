use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

// Domain types
#[derive(Clone, Deserialize, Serialize)]
pub struct Astronaut {
    pub id: String,
    pub name: String,
    pub birth_date: DateTime<Utc>,
}

// Input types
#[derive(Deserialize, Serialize)]
pub struct CreateAstronautInput {
    pub name: String,
    pub password: String,
    pub birth_date: DateTime<Utc>,
}

#[derive(Deserialize, Serialize)]
pub struct UpdateAstronautInput {
    pub name: Option<String>,
    pub password: Option<String>,
    pub birth_date: Option<DateTime<Utc>>,
}

impl UpdateAstronautInput {
    pub fn is_empty(&self) -> bool {
        self.name == None && self.password == None && self.birth_date == None
    }
}

// Output types
#[derive(Deserialize, Serialize)]
pub struct CreateAstronautOutput {
    pub id: String,
}

// Mongo types
#[derive(Clone, Deserialize, Serialize)]
pub struct AstronautDocument {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub birth_date: DateTime<Utc>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct AstronautUpdatedDocument {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birth_date: Option<DateTime<Utc>>,
}

// Kafka types
#[derive(Clone, Deserialize, Serialize)]
pub struct AstronautCreatedEvent {
    pub id: String,
    pub name: String,
    pub password: String,
    pub birth_date: DateTime<Utc>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct AstronautUpdatedEvent {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birth_date: Option<DateTime<Utc>>,
}

// Transformation between types
impl From<&AstronautDocument> for Astronaut {
    fn from(input: &AstronautDocument) -> Self {
        Self {
            id: input.id.clone(),
            name: input.name.clone(),
            birth_date: input.birth_date.clone(),
        }
    }
}

impl From<&AstronautCreatedEvent> for AstronautDocument {
    fn from(input: &AstronautCreatedEvent) -> Self {
        Self {
            id: input.id.clone(),
            name: input.name.clone(),
            birth_date: input.birth_date.clone(),
        }
    }
}

impl From<&AstronautUpdatedEvent> for AstronautUpdatedDocument {
    fn from(input: &AstronautUpdatedEvent) -> Self {
        Self {
            name: input.name.clone(),
            birth_date: input.birth_date.clone(),
        }
    }
}
