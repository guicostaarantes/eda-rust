use crate::domain::astronaut_model::Astronaut;
use crate::domain::astronaut_querier::AstronautQuerier;
use crate::domain::astronaut_querier::AstronautQuerierError;
use crate::providers::token::Token;
use async_graphql::Context;
use async_graphql::Object;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn check_astronaut_credentials(
        &self,
        ctx: &Context<'_>,
    ) -> Result<String, AstronautQuerierError> {
        match ctx.data_opt::<Token>() {
            Some(tok) => Ok(tok.content.clone()),
            None => Err(AstronautQuerierError::TokenNotFound),
        }
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
