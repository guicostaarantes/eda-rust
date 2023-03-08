use chrono::DateTime;
use chrono::Datelike;
use chrono::TimeZone;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Astronaut {
    #[serde(rename = "_id")]
    id: String,
    name: String,
    birth_date: DateTime<Utc>,
}

impl Astronaut {
    pub fn new(name: String, birth_date: DateTime<Utc>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            birth_date,
        }
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}

#[async_graphql::Object]
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
