use crate::domain::astronaut_model::Astronaut;
use crate::domain::astronaut_querier::AstronautQuerier;
use crate::domain::astronaut_querier::AstronautQuerierError;
use crate::providers::token::RawToken;
use async_graphql::Context;
use async_graphql::Object;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn check_astronaut_credentials(
        &self,
        ctx: &Context<'_>,
    ) -> Result<String, AstronautQuerierError> {
        match ctx.data_opt::<RawToken>() {
            Some(raw_token) => Ok(raw_token.0.clone()),
            None => Err(AstronautQuerierError::TokenNotFound),
        }
    }

    async fn astronaut_by_id(
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
            None => Err(AstronautQuerierError::TokenNotFound),
        }
    }
}
