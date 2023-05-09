mod mutation;
mod query;
mod subscription;

use async_graphql::Schema;
pub use mutation::Mutation;
pub use query::Query;
pub use subscription::Subscription;

pub type MissionsSchema = Schema<Query, Mutation, Subscription>;
