mod mutation;
mod query;

pub use async_graphql::EmptySubscription as SubscriptionRoot;
use async_graphql::Schema;
pub use mutation::MutationRoot;
pub use query::QueryRoot;

pub type AuthSchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;
