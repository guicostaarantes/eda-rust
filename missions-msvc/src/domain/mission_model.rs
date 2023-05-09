use async_graphql::futures_util::future::try_join_all;
use async_graphql::Context;
use async_graphql::InputObject;
use async_graphql::Object;
use async_graphql::ID;
use chrono::DateTime;
use chrono::Datelike;
use chrono::TimeZone;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

use crate::providers::token::RawToken;

use super::mission_querier::MissionQuerier;
use super::mission_querier::MissionQuerierError;

// Pure types for domain use
#[derive(Clone)]
pub struct Mission {
    pub id: String,
    pub name: String,
    pub start_date: DateTime<Utc>,
    pub crew_ids: Vec<String>,
}

#[derive(Clone, Default)]
pub struct Astronaut {
    pub id: String,
    pub participant_of_ids: Vec<String>,
}

// GraphQL fields
#[Object]
impl Mission {
    async fn id(&self) -> ID {
        ID(self.id.clone())
    }

    async fn name(&self) -> String {
        self.name.clone()
    }

    async fn duration(&self) -> String {
        let today = Utc::now();
        let years = today.years_since(self.start_date).unwrap();
        let remaining = Utc
            .with_ymd_and_hms(
                self.start_date.year() + years as i32,
                self.start_date.month(),
                self.start_date.day(),
                0,
                0,
                0,
            )
            .unwrap();
        let days = today.signed_duration_since(remaining).num_days();

        format!("{} years, {} days", years, days)
    }

    async fn crew(&self, ctx: &Context<'_>) -> Result<Vec<Astronaut>, MissionQuerierError> {
        match ctx.data_opt::<RawToken>() {
            Some(raw_token) => match try_join_all(self.crew_ids.iter().map(|id| {
                ctx.data_unchecked::<MissionQuerier>()
                    .get_astronaut_by_id(raw_token, id.clone())
            }))
            .await
            {
                Ok(result) => Ok(result),
                Err(_) => Err(MissionQuerierError::Forbidden),
            },
            None => Err(MissionQuerierError::Forbidden),
        }
    }
}

#[Object]
impl Astronaut {
    async fn id(&self) -> ID {
        ID(self.id.clone())
    }

    async fn participant_of(&self, ctx: &Context<'_>) -> Result<Vec<Mission>, MissionQuerierError> {
        match ctx.data_opt::<RawToken>() {
            Some(raw_token) => match try_join_all(self.participant_of_ids.iter().map(|id| {
                ctx.data_unchecked::<MissionQuerier>()
                    .get_mission_by_id(raw_token, id.clone())
            }))
            .await
            {
                Ok(result) => Ok(result),
                Err(_) => Err(MissionQuerierError::Forbidden),
            },
            None => Err(MissionQuerierError::Forbidden),
        }
    }
}

// GraphQL input types
#[derive(InputObject)]
pub struct CreateMissionInput {
    pub name: String,
    pub start_date: DateTime<Utc>,
}

#[derive(InputObject)]
pub struct UpdateMissionInput {
    pub name: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
}

impl UpdateMissionInput {
    pub fn is_empty(&self) -> bool {
        self.name == None && self.start_date == None
    }
}

#[derive(InputObject)]
pub struct AddCrewMemberToMissionInput {
    pub mission_id: String,
    pub astronaut_id: String,
}

#[derive(InputObject)]
pub struct RemoveCrewMemberFromMissionInput {
    pub mission_id: String,
    pub astronaut_id: String,
}

// Mongo types
#[derive(Clone, Deserialize, Serialize)]
pub struct MissionDocument {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub start_date: DateTime<Utc>,
    pub crew: Vec<String>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct AstronautDocument {
    #[serde(rename = "_id")]
    pub id: String,
    pub participant_of: Vec<String>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct MissionUpdateDocument {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<DateTime<Utc>>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct CrewMemberUpdateDocument {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crew: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub participant_of: Option<String>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct CrewMemberRemoveDocument {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crew: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub participant_of: Option<String>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct IdDocument {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

// Kafka types
#[derive(Clone, Deserialize, Serialize)]
pub struct MissionCreatedEvent {
    pub id: String,
    pub name: String,
    pub start_date: DateTime<Utc>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct MissionUpdatedEvent {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<DateTime<Utc>>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct CrewMemberAddedEvent {
    pub mission_id: String,
    pub astronaut_id: String,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct CrewMemberRemovedEvent {
    pub mission_id: String,
    pub astronaut_id: String,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct AstronautCreatedEvent {
    pub id: String,
}

// Transformation between types
impl From<&MissionDocument> for Mission {
    fn from(input: &MissionDocument) -> Self {
        Self {
            id: input.id.clone(),
            name: input.name.clone(),
            start_date: input.start_date.clone(),
            crew_ids: input.crew.clone(),
        }
    }
}

impl From<&AstronautDocument> for Astronaut {
    fn from(input: &AstronautDocument) -> Self {
        Self {
            id: input.id.clone(),
            participant_of_ids: input.participant_of.clone(),
        }
    }
}

impl From<&MissionCreatedEvent> for MissionDocument {
    fn from(input: &MissionCreatedEvent) -> Self {
        Self {
            id: input.id.clone(),
            name: input.name.clone(),
            start_date: input.start_date.clone(),
            crew: Vec::new(),
        }
    }
}

impl From<&AstronautCreatedEvent> for AstronautDocument {
    fn from(input: &AstronautCreatedEvent) -> Self {
        Self {
            id: input.id.clone(),
            participant_of: Vec::new(),
        }
    }
}

impl From<&MissionUpdatedEvent> for MissionUpdateDocument {
    fn from(input: &MissionUpdatedEvent) -> Self {
        Self {
            name: input.name.clone(),
            start_date: input.start_date.clone(),
        }
    }
}

impl Mission {
    pub fn apply_mission_updated_event(&self, event: &MissionUpdatedEvent) -> Self {
        Self {
            id: self.id.clone(),
            name: event.name.clone().unwrap_or(self.name.clone()),
            start_date: event.start_date.clone().unwrap_or(self.start_date.clone()),
            crew_ids: self.crew_ids.clone(),
        }
    }

    pub fn apply_crew_member_added_event(&self, event: &CrewMemberAddedEvent) -> Self {
        Self {
            id: self.id.clone(),
            name: self.name.clone(),
            start_date: self.start_date.clone(),
            crew_ids: {
                let mut ids = self.crew_ids.clone();
                ids.push(event.astronaut_id.clone());
                ids
            },
        }
    }

    pub fn apply_crew_member_removed_event(&self, event: &CrewMemberRemovedEvent) -> Self {
        Self {
            id: self.id.clone(),
            name: self.name.clone(),
            start_date: self.start_date.clone(),
            crew_ids: {
                let mut ids = self.crew_ids.clone();
                ids = ids
                    .into_iter()
                    .filter(|c| c != &event.astronaut_id)
                    .collect();
                ids
            },
        }
    }
}
