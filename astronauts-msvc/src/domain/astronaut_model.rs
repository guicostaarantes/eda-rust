use async_graphql::InputObject;
use async_graphql::Object;
use async_graphql::ID;
use chrono::DateTime;
use chrono::Datelike;
use chrono::TimeZone;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

// Pure types for domain use
#[derive(Clone)]
pub struct Astronaut {
    pub id: String,
    pub name: String,
    pub password: String,
    pub birth_date: DateTime<Utc>,
}

// GraphQL fields
#[Object]
impl Astronaut {
    async fn id(&self) -> ID {
        ID(self.id.clone())
    }

    async fn name(&self) -> String {
        self.name.clone()
    }

    async fn age(&self) -> String {
        let today = Utc::now();
        let years = today.years_since(self.birth_date).unwrap();
        let remaining = Utc
            .with_ymd_and_hms(
                self.birth_date.year() + years as i32,
                self.birth_date.month(),
                self.birth_date.day(),
                0,
                0,
                0,
            )
            .unwrap();
        let days = today.signed_duration_since(remaining).num_days();

        format!("{} years, {} days", years, days)
    }
}

// GraphQL input types
#[derive(InputObject)]
pub struct CreateAstronautInput {
    pub name: String,
    pub password: String,
    pub birth_date: DateTime<Utc>,
}

#[derive(InputObject)]
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

// Mongo types
#[derive(Clone, Deserialize, Serialize)]
pub struct AstronautDocument {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub password: String,
    pub birth_date: DateTime<Utc>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct AstronautUpdatedDocument {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
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
            password: input.password.clone(),
            birth_date: input.birth_date.clone(),
        }
    }
}

impl From<&AstronautCreatedEvent> for AstronautDocument {
    fn from(input: &AstronautCreatedEvent) -> Self {
        Self {
            id: input.id.clone(),
            name: input.name.clone(),
            password: input.password.clone(),
            birth_date: input.birth_date.clone(),
        }
    }
}

impl From<&AstronautUpdatedEvent> for AstronautUpdatedDocument {
    fn from(input: &AstronautUpdatedEvent) -> Self {
        Self {
            name: input.name.clone(),
            password: input.password.clone(),
            birth_date: input.birth_date.clone(),
        }
    }
}
