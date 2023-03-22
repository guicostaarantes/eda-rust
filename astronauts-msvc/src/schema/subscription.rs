use crate::domain::astronaut_model::Astronaut;
use crate::domain::astronaut_querier::AstronautQuerier;
use async_graphql::Context;
use async_graphql::Subscription;
use tokio_stream::Stream;

pub struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    async fn last_astronaut_created<'a>(
        &'a self,
        ctx: &Context<'a>,
    ) -> impl Stream<Item = Astronaut> + 'a {
        ctx.data_unchecked::<AstronautQuerier>()
            .subscribe_to_astronaut_created()
    }
}
