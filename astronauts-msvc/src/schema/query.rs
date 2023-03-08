use crate::domain::astronaut_controller::AstronautController;
use crate::domain::astronaut_controller::AstronautControllerError;
use crate::domain::Astronaut;
use async_graphql::Context;
use async_graphql::Object;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn astronaut_by_id(
        &self,
        ctx: &Context<'_>,
        id: String,
    ) -> Result<Option<Astronaut>, AstronautControllerError> {
        ctx.data_unchecked::<AstronautController>()
            .get_astronaut_by_id(id)
            .await
    }
}
