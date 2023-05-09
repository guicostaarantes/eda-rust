use crate::domain::astronaut_commander::AstronautCommander;
use crate::domain::astronaut_commander::AstronautCommanderError;
use crate::domain::astronaut_model::CreateAstronautInput;
use crate::domain::astronaut_model::UpdateAstronautInput;
use crate::providers::token::RawToken;
use async_graphql::Context;
use async_graphql::Object;

pub struct Mutation;

#[Object]
impl Mutation {
    async fn create_astronaut(
        &self,
        ctx: &Context<'_>,
        input: CreateAstronautInput,
    ) -> Result<String, AstronautCommanderError> {
        ctx.data_unchecked::<AstronautCommander>()
            .create_astronaut(input)
            .await
    }

    async fn update_astronaut(
        &self,
        ctx: &Context<'_>,
        id: String,
        input: UpdateAstronautInput,
    ) -> Result<String, AstronautCommanderError> {
        match ctx.data_opt::<RawToken>() {
            Some(raw_token) => {
                ctx.data_unchecked::<AstronautCommander>()
                    .update_astronaut(raw_token, id, input)
                    .await
            }
            None => Err(AstronautCommanderError::Forbidden),
        }
    }
}
