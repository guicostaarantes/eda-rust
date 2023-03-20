use async_graphql::InputObject;
use async_graphql::Object;
use chrono::DateTime;
use chrono::Datelike;
use chrono::TimeZone;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Deserialize, Serialize)]
pub struct Astronaut {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub birth_date: DateTime<Utc>,
}

#[Object]
impl Astronaut {
    async fn id(&self) -> String {
        self.id.clone()
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

#[derive(InputObject)]
pub struct CreateAstronautInput {
    pub name: String,
    pub birth_date: DateTime<Utc>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct AstronautCreatedEvent {
    #[serde(skip_serializing)]
    pub id: String,
    pub name: String,
    pub birth_date: DateTime<Utc>,
}

#[derive(InputObject)]
pub struct UpdateAstronautInput {
    pub name: Option<String>,
    pub birth_date: Option<DateTime<Utc>>,
}

impl UpdateAstronautInput {
    pub fn is_empty(&self) -> bool {
        self.name == None && self.birth_date == None
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct AstronautUpdatedEvent {
    #[serde(skip_serializing)]
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birth_date: Option<DateTime<Utc>>,
}
