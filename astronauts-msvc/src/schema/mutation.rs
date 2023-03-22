use crate::domain::astronaut_controller::AstronautController;
use crate::domain::astronaut_controller::AstronautControllerError;
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
    ) -> Result<String, AstronautControllerError> {
        ctx.data_unchecked::<AstronautController>()
            .create_astronaut(input)
            .await
    }

    async fn update_astronaut(
        &self,
        ctx: &Context<'_>,
        id: String,
        input: UpdateAstronautInput,
    ) -> Result<String, AstronautControllerError> {
        ctx.data_unchecked::<AstronautController>()
            .update_astronaut(id, input)
            .await
    }

    async fn get_astronaut_credentials(
        &self,
        ctx: &Context<'_>,
        input: GetAstronautCredentialsInput,
    ) -> Result<String, AstronautControllerError> {
        ctx.data_unchecked::<AstronautController>()
            .get_astronaut_credentials(input)
            .await
    }
}
