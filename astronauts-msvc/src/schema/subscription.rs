use crate::domain::astronaut_model::Astronaut;
use crate::domain::astronaut_querier::AstronautQuerier;
use crate::domain::astronaut_querier::AstronautQuerierError;
use crate::providers::token::RawToken;
use async_graphql::Context;
use async_graphql::Subscription;
use tokio_stream::Stream;

pub struct Subscription;

#[Subscription]
impl Subscription {
    async fn astronaut(
        &self,
        ctx: &Context<'_>,
        id: String,
    ) -> Result<impl Stream<Item = Astronaut>, AstronautQuerierError> {
        match ctx.data_opt::<RawToken>() {
            Some(raw_token) => {
                ctx.data_unchecked::<AstronautQuerier>()
                    .get_astronaut_by_id_stream(raw_token, id)
                    .await
            }
            None => Err(AstronautQuerierError::Forbidden),
        }
    }
}
