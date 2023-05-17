use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

// Domain types
#[derive(Clone, Deserialize, Serialize)]
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

// Input types
#[derive(Deserialize, Serialize)]
pub struct CreateMissionInput {
    pub name: String,
    pub start_date: DateTime<Utc>,
}

#[derive(Deserialize, Serialize)]
pub struct UpdateMissionInput {
    pub name: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
}

impl UpdateMissionInput {
    pub fn is_empty(&self) -> bool {
        self.name == None && self.start_date == None
    }
}

#[derive(Deserialize, Serialize)]
pub struct AddCrewMemberToMissionInput {
    pub mission_id: String,
    pub astronaut_id: String,
}

#[derive(Deserialize, Serialize)]
pub struct RemoveCrewMemberFromMissionInput {
    pub mission_id: String,
    pub astronaut_id: String,
}

// Output types
#[derive(Deserialize, Serialize)]
pub struct CreateMissionOutput {
    pub id: String,
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
