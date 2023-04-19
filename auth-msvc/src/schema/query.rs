use async_graphql::Context;
use async_graphql::Object;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn noop(
        &self,
        ctx: &Context<'_>,
    ) -> String {
        "ok".to_string()
    }
}
