use crate::domain::mission_commander::MissionCommander;
use crate::domain::mission_commander::MissionCommanderError;
use crate::domain::mission_model::CreateMissionInput;
use crate::domain::mission_model::UpdateMissionInput;
use crate::providers::token::RawToken;
use async_graphql::Context;
use async_graphql::Object;

pub struct Mutation;

#[Object]
impl Mutation {
    async fn create_mission(
        &self,
        ctx: &Context<'_>,
        input: CreateMissionInput,
    ) -> Result<String, MissionCommanderError> {
        match ctx.data_opt::<RawToken>() {
            Some(raw_token) => {
                ctx.data_unchecked::<MissionCommander>()
                    .create_mission(raw_token, input)
                    .await
            }
            None => Err(MissionCommanderError::Forbidden),
        }
    }

    async fn update_mission(
        &self,
        ctx: &Context<'_>,
        id: String,
        input: UpdateMissionInput,
    ) -> Result<String, MissionCommanderError> {
        match ctx.data_opt::<RawToken>() {
            Some(raw_token) => {
                ctx.data_unchecked::<MissionCommander>()
                    .update_mission(raw_token, id, input)
                    .await
            }
            None => Err(MissionCommanderError::Forbidden),
        }
    }
}
