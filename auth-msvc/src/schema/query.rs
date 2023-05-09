use async_graphql::Context;
use async_graphql::Object;

pub struct Query;

#[Object]
impl Query {
    async fn noop(&self, _ctx: &Context<'_>) -> String {
        "ok".to_string()
    }
}
