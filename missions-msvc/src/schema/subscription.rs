use crate::domain::mission_model::Mission;
use crate::domain::mission_querier::MissionQuerier;
use crate::domain::mission_querier::MissionQuerierError;
use crate::providers::token::RawToken;
use async_graphql::Context;
use async_graphql::Subscription;
use tokio_stream::Stream;

pub struct Subscription;

#[Subscription]
impl Subscription {
    async fn mission_by_id(
        &self,
        ctx: &Context<'_>,
        id: String,
    ) -> Result<impl Stream<Item = Mission>, MissionQuerierError> {
        match ctx.data_opt::<RawToken>() {
            Some(raw_token) => {
                ctx.data_unchecked::<MissionQuerier>()
                    .get_mission_by_id_stream(raw_token, id)
                    .await
            }
            None => Err(MissionQuerierError::Forbidden),
        }
    }
}
