use crate::domain::astronaut_model::Astronaut;
use crate::domain::astronaut_querier::AstronautQuerier;
use crate::domain::astronaut_querier::AstronautQuerierError;
use async_graphql::Context;
use async_graphql::Object;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn check_astronaut_credentials(
        &self,
        ctx: &Context<'_>,
        token: String,
    ) -> Result<String, AstronautQuerierError> {
        ctx.data_unchecked::<AstronautQuerier>()
            .check_astronaut_credentials(token)
            .await
    }

    async fn astronaut_by_id(
        &self,
        ctx: &Context<'_>,
        id: String,
    ) -> Result<Astronaut, AstronautQuerierError> {
        ctx.data_unchecked::<AstronautQuerier>()
            .get_astronaut_by_id(id)
            .await
    }
}
