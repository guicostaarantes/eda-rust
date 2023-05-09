use crate::domain::astronaut_model::Astronaut;
use crate::domain::astronaut_querier::AstronautQuerier;
use crate::domain::astronaut_querier::AstronautQuerierError;
use crate::providers::token::RawToken;
use async_graphql::Context;
use async_graphql::Object;

pub struct Query;

#[Object]
impl Query {
    async fn astronaut(
        &self,
        ctx: &Context<'_>,
        id: String,
    ) -> Result<Astronaut, AstronautQuerierError> {
        match ctx.data_opt::<RawToken>() {
            Some(raw_token) => {
                ctx.data_unchecked::<AstronautQuerier>()
                    .get_astronaut_by_id(raw_token, id)
                    .await
            }
            None => Err(AstronautQuerierError::Forbidden),
        }
    }

    #[graphql(entity)]
    async fn astronaut_by_id(&self, ctx: &Context<'_>, #[graphql(key)] id: String) -> Astronaut {
        match ctx.data_opt::<RawToken>() {
            Some(raw_token) => {
                match ctx
                    .data_unchecked::<AstronautQuerier>()
                    .get_astronaut_by_id(raw_token, id)
                    .await
                {
                    Ok(astronaut) => astronaut,
                    Err(_) => Astronaut::default(),
                }
            }
            None => Astronaut::default(),
        }
    }
}
