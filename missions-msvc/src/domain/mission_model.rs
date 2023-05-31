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
}

#[derive(Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MissionRole {
    Leader,
    Member,
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
pub struct AddCrewInput {
    #[serde(default)]
    pub mission_id: String,
    pub astronaut_id: String,
    pub roles: Vec<MissionRole>,
}

#[derive(Deserialize, Serialize)]
pub struct UpdateCrewInput {
    #[serde(default)]
    pub mission_id: String,
    pub astronaut_id: String,
    pub roles: Vec<MissionRole>,
}

#[derive(Deserialize, Serialize)]
pub struct RemoveCrewInput {
    #[serde(default)]
    pub mission_id: String,
    pub astronaut_id: String,
}

// Output types
#[derive(Deserialize, Serialize)]
pub struct CreateMissionOutput {
    pub id: String,
}

#[derive(Deserialize, Serialize)]
pub struct MissionCrewInfoItem {
    pub astronaut_id: String,
    pub roles: Vec<MissionRole>,
}

#[derive(Default, Deserialize, Serialize)]
pub struct MissionCrewInfo(pub Vec<MissionCrewInfoItem>);

#[derive(Deserialize, Serialize)]
pub struct AstronautCrewInfoItem {
    pub mission_id: String,
    pub roles: Vec<MissionRole>,
}

#[derive(Default, Deserialize, Serialize)]
pub struct AstronautCrewInfo(pub Vec<AstronautCrewInfoItem>);

// Mongo types
#[derive(Clone, Deserialize, Serialize)]
pub struct MissionDocument {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub start_date: DateTime<Utc>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct MissionUpdateDocument {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<DateTime<Utc>>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct CrewDocument {
    #[serde(rename = "_id")]
    pub id: String,
    pub mission_id: String,
    pub astronaut_id: String,
    pub roles: Vec<MissionRole>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct CrewUpdateDocument {
    pub roles: Vec<MissionRole>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct CrewRemoveDocument {
    pub mission_id: String,
    pub astronaut_id: String,
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
pub struct CrewUpdatedEvent {
    pub mission_id: String,
    pub astronaut_id: String,
    pub roles: Vec<MissionRole>,
}

// Transformation between types
impl From<&MissionDocument> for Mission {
    fn from(input: &MissionDocument) -> Self {
        Self {
            id: input.id.clone(),
            name: input.name.clone(),
            start_date: input.start_date.clone(),
        }
    }
}

impl From<&MissionCreatedEvent> for MissionDocument {
    fn from(input: &MissionCreatedEvent) -> Self {
        Self {
            id: input.id.clone(),
            name: input.name.clone(),
            start_date: input.start_date.clone(),
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

impl From<&Vec<CrewDocument>> for MissionCrewInfo {
    fn from(input: &Vec<CrewDocument>) -> Self {
        input
            .iter()
            .fold(MissionCrewInfo::default(), |mut acc, val| {
                acc.0.push(MissionCrewInfoItem {
                    astronaut_id: val.astronaut_id.clone(),
                    roles: val.roles.clone(),
                });
                acc
            })
    }
}

impl From<&Vec<CrewDocument>> for AstronautCrewInfo {
    fn from(input: &Vec<CrewDocument>) -> Self {
        input
            .iter()
            .fold(AstronautCrewInfo::default(), |mut acc, val| {
                acc.0.push(AstronautCrewInfoItem {
                    mission_id: val.mission_id.clone(),
                    roles: val.roles.clone(),
                });
                acc
            })
    }
}
