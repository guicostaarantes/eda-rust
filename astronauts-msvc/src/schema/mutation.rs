use crate::domain::astronaut_controller::AstronautController;
use crate::domain::astronaut_controller::AstronautControllerError;
use crate::domain::Astronaut;
use async_graphql::Context;
use async_graphql::Object;
use chrono::DateTime;
use chrono::Utc;

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn create_astronaut(
        &self,
        ctx: &Context<'_>,
        name: String,
        birth_date: DateTime<Utc>,
    ) -> Result<Astronaut, AstronautControllerError> {
        ctx.data_unchecked::<AstronautController>()
            .create_astronaut(Astronaut::new(name, birth_date))
            .await
    }
}
