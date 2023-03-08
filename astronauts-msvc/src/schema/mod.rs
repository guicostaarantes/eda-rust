mod mutation;
mod query;
mod subscription;

use async_graphql::Schema;
pub use mutation::MutationRoot;
pub use query::QueryRoot;
pub use subscription::SubscriptionRoot;

pub type AstronautsSchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;
