use crate::domain::mission_model::Astronaut;
use crate::domain::mission_model::Mission;
use crate::domain::mission_querier::MissionQuerier;
use crate::domain::mission_querier::MissionQuerierError;
use crate::providers::token::RawToken;
use async_graphql::Context;
use async_graphql::Object;

pub struct Query;

#[Object]
impl Query {
    async fn mission(&self, ctx: &Context<'_>, id: String) -> Result<Mission, MissionQuerierError> {
        match ctx.data_opt::<RawToken>() {
            Some(raw_token) => {
                ctx.data_unchecked::<MissionQuerier>()
                    .get_mission_by_id(raw_token, id)
                    .await
            }
            None => Err(MissionQuerierError::Forbidden),
        }
    }

    #[graphql(entity)]
    async fn astronaut_by_id(&self, ctx: &Context<'_>, #[graphql(key)] id: String) -> Astronaut {
        match ctx.data_opt::<RawToken>() {
            Some(raw_token) => {
                match ctx
                    .data_unchecked::<MissionQuerier>()
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
