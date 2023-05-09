mod mutation;
mod query;

pub use async_graphql::EmptySubscription as Subscription;
use async_graphql::Schema;
pub use mutation::Mutation;
pub use query::Query;

pub type AuthSchema = Schema<Query, Mutation, Subscription>;
