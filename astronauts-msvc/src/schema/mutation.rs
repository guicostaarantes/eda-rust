use crate::domain::astronaut_commander::AstronautCommander;
use crate::domain::astronaut_commander::AstronautCommanderError;
use crate::domain::astronaut_model::CreateAstronautInput;
use crate::domain::astronaut_model::GetAstronautCredentialsInput;
use crate::domain::astronaut_model::UpdateAstronautInput;
use async_graphql::Context;
use async_graphql::Object;

pub struct MutationRoot;

#[Object]
impl MutationRoot {
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
        ctx.data_unchecked::<AstronautCommander>()
            .update_astronaut(id, input)
            .await
    }

    async fn get_astronaut_credentials(
        &self,
        ctx: &Context<'_>,
        input: GetAstronautCredentialsInput,
    ) -> Result<String, AstronautCommanderError> {
        ctx.data_unchecked::<AstronautCommander>()
            .get_astronaut_credentials(input)
            .await
    }
}
